use crate::framework::{
    logger::{Logger, LoggerInterface},
    session::Session,
    AppState, ReqScopedState,
};
use axum::{extract::State, Json};
use serde::Serialize;

/// パス
pub const PATH: &'static str = "/";

pub async fn handler(
    State(state): State<AppState>,
    ctx: ReqScopedState,
    Session { user }: Session,
    logger: Logger,
) -> Json<ResponseValue> {
    logger.info("hello world");

    let resp_value = ResponseValue {
        greeting: format!("hello, {}!", user.name),
    };

    Json(resp_value)
}

#[derive(Serialize)]
pub struct ResponseValue {
    greeting: String,
}
