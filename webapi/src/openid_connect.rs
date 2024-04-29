use crate::{framework::AppState, settings::OPENID_CONNECT_STATE_KEY};
use axum::{
    extract::{self, Query},
    http::StatusCode,
    response::{ErrorResponse, IntoResponse, Redirect, Response},
};
use axum::{routing, Router};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use serde::Deserialize;
use serde_json::Value;

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
    let csrf_token = ulid::Ulid::new().to_string();

    let mut cookie = Cookie::new(OPENID_CONNECT_STATE_KEY, csrf_token.clone());
    cookie.set_secure(true);
    cookie.set_http_only(true);

    let jar = jar.add(cookie);
    let authorization_endpoint =
        reqwest::get("https://accounts.google.com/.well-known/openid-configuration")
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "a"))?
            .json::<Value>()
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "b"))?
            .get("authorization_endpoint")
            .and_then(|v| v.as_str().map(|x| x.to_string()))
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    let nonce = ulid::Ulid::new().to_string();

    let query_params: Vec<(&str, &str)> = vec![
        ("state", &csrf_token),
        ("client_id", &state.env.google_client_id),
        ("response_type", "code"),
        ("scope", "openid email"),
        ("redirect_uri", &state.env.google_redirect_uri),
        ("nonce", &nonce),
    ];

    let client_redirct_url =
        reqwest::Url::parse_with_params(&authorization_endpoint, &query_params)
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "b"))?;

    Ok((jar, Redirect::to(&client_redirct_url.to_string())).into_response())
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
    let state = if let Some(v) = jar.get(OPENID_CONNECT_STATE_KEY) {
        v
    } else {
        return Err(StatusCode::UNAUTHORIZED.into());
    };

    if state.value() != params.state {
        return Err(StatusCode::UNAUTHORIZED.into());
    }

    let new_jar = jar.clone().remove(state.clone());

    // TODO: codeの検証

    Ok((new_jar, Redirect::to("http://localhost:3000/login")).into_response())
}
