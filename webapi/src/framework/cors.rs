use axum::http::HeaderValue;
use tower_http::cors::CorsLayer;

use crate::settings::CORS_ALLOWED_ORIGINS;

pub fn mk_cors_layer() -> CorsLayer {
    CorsLayer::new().allow_origin(CORS_ALLOWED_ORIGINS.map(|origin| {
        origin
            .parse::<HeaderValue>()
            .expect("originをパースできる必要がある")
    }))
}
