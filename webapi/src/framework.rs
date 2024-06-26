pub mod env;
pub mod logger;
pub mod session;
pub mod system;
use self::env::Env;
use axum::{
    async_trait, extract,
    http::{request::Parts, StatusCode},
};
use chrono::{DateTime, Utc};
use sea_orm::DatabaseConnection;
use serde_json::Value;
use std::fmt::Debug;
use std::ops::Deref;
use ulid::Ulid;

/// アプリケーション全体での共有する状態. DBコネクションなどを持たせる.
#[derive(Clone)]
pub struct AppState {
    pub db_client: DatabaseConnection,
    pub env: Immutable<Env>,
    pub discovery_json: Value,
    // pub jwk_set: JwkSet,
}

/// リクエストごとに分離された状態.
#[derive(Clone, Debug)]
pub struct ReqScopedState {
    pub ts: DateTime<Utc>,
    pub req_id: Ulid,
}

impl ReqScopedState {
    pub fn new(req_id: Ulid) -> Self {
        let ts = DateTime::from_timestamp_millis(req_id.timestamp_ms() as i64).unwrap();

        Self { req_id, ts }
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

pub struct Meta<'a, Data, CustomProps = NoneOfProps> {
    pub data: &'a Data,
    pub version_tag: Ulid,
    pub custom_props: CustomProps,
}

pub struct NoneOfProps;

#[derive(Debug, Clone)]
pub struct Immutable<T>(T);

impl<T> Deref for Immutable<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
