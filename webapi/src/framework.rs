pub mod logger;

use axum::{
    async_trait, extract,
    http::{request::Parts, StatusCode},
    response::{self, IntoResponse},
};
use chrono::{DateTime, Utc};
use sea_orm::DatabaseConnection;
use serde_json::{json, Map, Value};
use std::net::SocketAddr;
use std::{error::Error, fmt::Debug};
use ulid::Ulid;

/// アプリケーション全体での共有する状態. DBコネクションなどを持たせる.
#[derive(Clone)]
pub struct AppState {
    pub db_client: DatabaseConnection,
    pub env: Env,
    pub discovery_json: Value,
    // pub jwk_set: JwkSet,
}

#[derive(Clone)]
pub struct Env {
    pub google_client_id: String,
    pub google_redirect_uri: String,
    pub google_client_secret: String,
}

impl ReqScopedState {
    pub fn logger(&self) -> logger::Logger {
        logger::Logger(self)
    }
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
impl<S> extract::FromRequestParts<S> for ReqScopedState
where
    S: Send + Sync,
    // AppState: FromRef<S>,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<ReqScopedState>()
            .cloned()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

#[derive(Debug)]
pub enum AppError {
    Unexpected(Box<dyn Error>),
    Unauthorized(Option<String>),
}

impl IntoResponse for AppError {
    fn into_response(self) -> response::Response {
        todo!()
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Unexpected(str) => {
                write!(f, "{}", str)
            }
            AppError::Unauthorized(msg) => {
                let msg = msg.clone();
                write!(f, "Unauthorized: {}", msg.unwrap_or("no message".into()))
            }
        }
    }
}

impl Error for AppError {}

/// ユーザー
pub enum User {
    /// 認証済みユーザー
    Authenticated(AuthenticatedUser),
    /// 認証されていないユーザー
    Anonymous,
}

/// 認証済みユーザー
#[derive(Clone, Debug)]
pub struct AuthenticatedUser {
    pub id: String,
    pub roles: Vec<Role>,
    pub name: String,
}

/// 役割
#[derive(Clone, Debug)]
pub enum Role {
    General,
    Admin,
    Master,
}

/// セッション
#[derive(Clone, Debug)]
pub struct Session {
    pub user: AuthenticatedUser,
}

// ハンドラの引数で指定できるようにするための処理
#[async_trait]
impl<S> extract::FromRequestParts<S> for Session
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<ReqScopedState>()
            .and_then(|item| item.session.clone())
            .ok_or(StatusCode::UNAUTHORIZED)
    }
}

/// sessionを探す
pub async fn find_session(str: &str) -> Option<Session> {
    if str == "xxx" {
        return Some(Session {
            user: AuthenticatedUser {
                id: "xxx".to_string(),
                roles: vec![Role::General],
                name: "takuya".to_string(),
            },
        });
    }
    None
}
