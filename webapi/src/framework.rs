pub mod env;
pub mod logger;
pub mod session;
pub mod system;

use self::{env::Env, session::Session};
use axum::{
    async_trait, extract,
    http::{request::Parts, StatusCode},
};
use chrono::{DateTime, Utc};
use sea_orm::DatabaseConnection;
use serde_json::{json, Map, Value};
use std::fmt::Debug;
use std::net::SocketAddr;
use ulid::Ulid;

/// アプリケーション全体での共有する状態. DBコネクションなどを持たせる.
#[derive(Clone)]
pub struct AppState {
    pub db_client: DatabaseConnection,
    pub env: Env,
    pub discovery_json: Value,
    // pub jwk_set: JwkSet,
}

/// リクエストごとに分離された状態.
#[derive(Clone, Debug)]
pub struct ReqScopedState {
    pub ts: DateTime<Utc>,
    pub req_id: Ulid,
    pub session: Option<Session>,
    pub log_member: Map<String, Value>,
}

impl ReqScopedState {
    pub fn new(
        req_id: Ulid,
        session: Option<Session>,
        req: &extract::Request,
        remote_addr: &SocketAddr,
    ) -> Self {
        let method = req.method();
        let uri = req.uri();

        let ts = DateTime::from_timestamp_millis(req_id.timestamp_ms() as i64).unwrap();
        let mut pairs = vec![
            ("req_id", req_id.to_string()),
            ("timestamp", ts.to_utc().to_rfc3339()),
            ("uri", uri.to_string()),
            ("method", method.to_string()),
            ("remote_addr", remote_addr.to_string()),
        ];

        let header_keys = vec!["user-agent", "cookie"];

        for key in header_keys {
            if let Some(v) = req.headers().get(key) {
                pairs.push((key, v.to_str().unwrap_or("parse error").to_string()));
            }
        }

        Self {
            req_id,
            session,
            ts,
            log_member: Map::from_iter(pairs.iter().map(|(k, v)| (k.to_string(), json!(v)))).into(),
        }
    }
}

#[async_trait]
impl<S: Send + Sync> extract::FromRequestParts<S> for ReqScopedState {
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<ReqScopedState>()
            .cloned()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

impl ReqScopedState {
    pub fn logger(&self) -> logger::Logger {
        logger::Logger(self)
    }
}
