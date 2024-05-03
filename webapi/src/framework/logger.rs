use std::net::SocketAddr;

use super::ReqScopedState;
use axum::{
    async_trait, extract,
    http::{request::Parts, StatusCode},
};
use serde_json::{json, Map, Value};

pub trait LoggerInterface {
    fn info(&self, item: &str);
    fn warning(&self, item: &str);
    fn danger(&self, item: &str);
    fn debug(&self, item: &str);
}

#[derive(Clone, Debug)]

pub struct Logger(Map<String, Value>);

impl Logger {
    pub fn new(ctx: &ReqScopedState, req: &extract::Request, remote_addr: &SocketAddr) -> Self {
        let method = req.method();
        let uri = req.uri();
        let mut pairs = vec![
            ("req_id", ctx.req_id.to_string()),
            ("timestamp", ctx.ts.to_utc().to_rfc3339()),
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

        Logger(Map::from_iter(pairs.iter().map(|(k, v)| (k.to_string(), json!(v)))).into())
    }

    fn log(mut map: Map<String, Value>, level: LogLevel, item: &str) {
        map.insert("log_level".to_string(), json!(level.to_string()));
        map.insert("message".to_string(), json!(item.to_string()));

        let tmp: Value = map.into();
        println!("{tmp}")
    }
}

impl LoggerInterface for Logger {
    fn info(&self, item: &str) {
        Logger::log(self.0.clone(), LogLevel::Info, item)
    }
    fn warning(&self, item: &str) {
        Logger::log(self.0.clone(), LogLevel::Warning, item)
    }

    fn danger(&self, item: &str) {
        Logger::log(self.0.clone(), LogLevel::Danger, item)
    }

    fn debug(&self, item: &str) {
        Logger::log(self.0.clone(), LogLevel::Debug, item)
    }
}

#[async_trait]
impl<S: Send + Sync> extract::FromRequestParts<S> for Logger {
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Logger>()
            .cloned()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)
    }
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
