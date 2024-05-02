use std::{error::Error, fmt::Debug, marker::PhantomData};

use axum::{
    async_trait,
    body::Body,
    extract::{self, ConnectInfo},
    http::{request::Parts, Response, StatusCode},
    response::{self, ErrorResponse, IntoResponse},
};
use chrono::{DateTime, Utc};
use jsonwebtoken::jwk::JwkSet;
use sea_orm::DatabaseConnection;
use serde_json::{json, Map, Value};
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

#[derive(Clone)]
pub struct Env {
    pub google_client_id: String,
    pub google_redirect_uri: String,
    pub google_client_secret: String,
}

#[derive(Clone, Debug)]
enum LogLevel {
    Info,
    Warning,
    Danger,
    Debug,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let item = match self {
            LogLevel::Info => "info",
            LogLevel::Warning => "warning",
            LogLevel::Danger => "danger",
            LogLevel::Debug => "debug",
        };
        write!(f, "{}", item)
    }
}

#[derive(Clone, Debug)]

pub struct Logger<'a>(&'a ReqScopedState);

impl ReqScopedState {
    pub fn logger(&self) -> Logger {
        Logger(self)
    }
}

pub trait LoggerInterface {
    fn info(&self, item: &str);
    fn warning(&self, item: &str);
    fn danger(&self, item: &str);
    fn debug(&self, item: &str);
}

impl<'a> LoggerInterface for Logger<'a> {
    fn info(&self, item: &str) {
        log(&self.0, LogLevel::Info, item)
    }
    fn warning(&self, item: &str) {
        log(&self.0, LogLevel::Warning, item)
    }

    fn danger(&self, item: &str) {
        log(&self.0, LogLevel::Danger, item)
    }

    fn debug(&self, item: &str) {
        log(&self.0, LogLevel::Debug, item)
    }
}

fn log(ctx: &ReqScopedState, level: LogLevel, item: &str) {
    let mut map = ctx.log_member.clone();

    map.insert("log_level".to_string(), json!(level.to_string()));
    map.insert("message".to_string(), json!(item.to_string()));

    let tmp: Value = map.into();
    println!("{tmp}")
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
                println!("Unexpected: {str}");
            }
            AppError::Unauthorized(msg) => {
                let msg = msg.clone();
                println!("Unauthorized: {}", msg.unwrap_or("no message".into()));
            }
        }

        Ok(())
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
    // AppState: FromRef<S>,
{
    type Rejection = Response<Body>;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let ctx = if let Some(item) = parts.extensions.get::<ReqScopedState>() {
            item.clone()
        } else {
            panic!("contextが未設定です")
        };

        // let app_state = AppState::from_ref(state);

        ctx.session
            .ok_or((StatusCode::UNAUTHORIZED, "ハズレ").into_response())
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
