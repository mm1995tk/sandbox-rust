use crate::framework::{AppState, Session};
use axum::{
    extract,
    response::{self, IntoResponse},
};
use serde::Serialize;

/// パス
pub const PATH: &'static str = "/";

pub async fn handler(
    extract::State(state): extract::State<AppState>,
    Session { user }: Session,
) -> impl IntoResponse {
    let resp_value = ResponseValue {
        greeting: format!("hello, {}!", user.name),
    };

    response::Json(resp_value)
}

#[derive(Serialize)]
struct ResponseValue {
    greeting: String,
}
