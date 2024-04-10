pub mod example_route {
    use axum::{extract, routing, Router};
    use crate::framework::{AppState, Session};

    /// パス
    pub const PATH: &'static str = "/example";

    pub fn mk_router() -> Router<AppState> {
        Router::new().route("/", routing::get(handler))
    }

    async fn handler(
        extract::State(state): extract::State<AppState>,
        Session { user }: Session,
    ) -> String {
        "Hello, world!".to_string()
    }
}

