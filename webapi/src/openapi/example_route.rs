use crate::framework::AppState;
use axum::{routing, Router};

mod root;

/// パス
pub const PATH: &str = "/example";

pub fn mk_router() -> Router<AppState> {
    Router::new().route(root::PATH, routing::get(root::handler))
}
