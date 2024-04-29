use crate::framework::AppState;
use axum::{
    extract,
    http::StatusCode,
    response::{self, IntoResponse},
};
use axum::{routing, Router};

/// パス
pub const PATH: &'static str = "/openid-connect";

pub fn mk_router() -> Router<AppState> {
    Router::new()
        .route("/", routing::get(handler))
        .route("/callback", routing::get(callback_handler))
}

pub async fn handler(extract::State(state): extract::State<AppState>) -> impl IntoResponse {
    let csrf_token = ulid::Ulid::new().to_string();
    StatusCode::NO_CONTENT
}

pub async fn callback_handler(
    extract::State(state): extract::State<AppState>,
) -> impl IntoResponse {
    StatusCode::NO_CONTENT
}
