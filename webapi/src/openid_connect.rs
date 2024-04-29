use crate::{db::openid_connect_states, framework::AppState, settings::OPENID_CONNECT_STATE_KEY};
use axum::{
    extract,
    http::StatusCode,
    response::{self, ErrorResponse, IntoResponse, Redirect, Response},
    Error,
};
use axum::{routing, Router};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use reqwest::{Client, RequestBuilder};
use sea_orm::{ActiveModelTrait, Set};
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
    let csrf_token = ulid::Ulid::new().to_string();
    let sid = ulid::Ulid::new().to_string();

    let t = openid_connect_states::ActiveModel {
        sid: Set(sid.clone()),
        state: Set(csrf_token.clone()),
    };

    t.insert(&state.db_client)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let jar = jar.add(Cookie::new(OPENID_CONNECT_STATE_KEY, sid));
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

async fn callback_handler(extract::State(state): extract::State<AppState>) -> impl IntoResponse {
    println!("kkkkkkkkdddd");
    StatusCode::NO_CONTENT
}
