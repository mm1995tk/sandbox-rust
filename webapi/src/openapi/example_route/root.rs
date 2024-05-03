use crate::framework::{
    logger::{Logger, LoggerInterface},
    session::Session,
    AppState, ReqScopedState,
};
use axum::{
    extract,
    response::{self, IntoResponse},
};
use serde::Serialize;

/// パス
pub const PATH: &'static str = "/";

pub async fn handler(
    extract::State(state): extract::State<AppState>,
    ctx: ReqScopedState,
    Session { user }: Session,
    logger: Logger,
) -> impl IntoResponse {
    logger.info("hello world");

    let resp_value = ResponseValue {
        greeting: format!("hello, {}!", user.name),
    };

    response::Json(resp_value)
}

#[derive(Serialize)]
struct ResponseValue {
    greeting: String,
}
