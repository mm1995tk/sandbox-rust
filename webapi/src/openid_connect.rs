use crate::{framework::AppState, settings::OPENID_CONNECT_STATE_KEY};
use axum::{
    extract::{self, Query},
    http::StatusCode,
    response::{ErrorResponse, IntoResponse, Redirect, Response},
};
use axum::{routing, Router};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use serde::Deserialize;
use serde_json::{json, Value};

/// パス
pub const PATH: &'static str = "/openid-connect";

pub fn mk_router() -> Router<AppState> {
    Router::new()
        .route("/", routing::get(handler))
        .route("/callback", routing::get(callback_handler))
}

async fn handler(
    extract::State(state): extract::State<AppState>,
    jar: CookieJar,
) -> Result<Response, ErrorResponse> {
    let authorization_endpoint = state
        .discovery_json
        .get("authorization_endpoint")
        .and_then(|v| v.as_str())
        .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "c"))?;

    let csrf_token = ulid::Ulid::new().to_string();
    let nonce = ulid::Ulid::new().to_string();

    let query_params: Vec<(&str, &str)> = vec![
        ("state", &csrf_token),
        ("client_id", &state.env.google_client_id),
        ("response_type", "code"),
        ("scope", "openid email"),
        ("redirect_uri", &state.env.google_redirect_uri),
        ("nonce", &nonce),
        ("access_type", "offline")
    ];

    let client_redirct_url = reqwest::Url::parse_with_params(authorization_endpoint, &query_params)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "d"))?;

    let redirect = Redirect::to(client_redirct_url.as_str());

    let jar = jar.add({
        let mut cookie = Cookie::new(OPENID_CONNECT_STATE_KEY, csrf_token);
        cookie.set_secure(true);
        cookie.set_http_only(true);
        cookie
    });
    Ok((jar, redirect).into_response())
}

#[derive(Deserialize)]
struct Params {
    code: String,
    state: String,
}
async fn callback_handler(
    Query(params): Query<Params>,
    extract::State(app_state): extract::State<AppState>,
    jar: CookieJar,
) -> Result<Response, ErrorResponse> {
    let state = match jar.get(OPENID_CONNECT_STATE_KEY) {
        Some(state) if state.value() == params.state => state.clone(),
        _ => {
            let err_response = (StatusCode::UNAUTHORIZED, "error");
            return Err(err_response.into());
        }
    };

    let token_endpoint = app_state
        .discovery_json
        .get("token_endpoint")
        .and_then(|v| v.as_str())
        .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "c"))?;

    let client = reqwest::Client::new();
    let body = json!({
        "code": params.code,
        "client_id": &app_state.env.google_client_id,
        "client_secret": &app_state.env.google_client_secret,
        "redirect_uri": &app_state.env.google_redirect_uri,
        "grant_type": "authorization_code"
    });
    let res = client
        .post(token_endpoint)
        .body(body.to_string())
        .send()
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "d"))?;

    let tokens = res
        .json::<Value>()
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "d"))?;

    println!("{}", tokens.to_string());

    // TODO: codeの検証

    let response = (jar.remove(state), Redirect::to("/login"));
    Ok(response.into_response())
}
