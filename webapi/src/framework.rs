use axum::{
    async_trait,
    body::Body,
    extract,
    http::{request::Parts, Response, StatusCode},
    response::IntoResponse,
};
use chrono::{DateTime, Utc};
use sea_orm::DatabaseConnection;
use ulid::Ulid;

/// アプリケーション全体での共有する状態. DBコネクションなどを持たせる.
#[derive(Clone)]
pub struct AppState {
    pub db_client: DatabaseConnection,
    pub env: Env,
}

#[derive(Clone)]
pub struct Env {
    pub google_client_id: String,
    pub google_redirect_uri: String,
}

/// リクエストごとに分離された状態.
#[derive(Clone, Debug)]
pub struct ReqScopedState {
    pub ts: DateTime<Utc>,
    pub req_id: Ulid,
    pub session: Option<Session>,
}

impl ReqScopedState {
    pub fn new(req_id: Ulid, session: Option<Session>) -> Self {
        Self {
            req_id,
            session,
            ts: DateTime::from_timestamp_millis(req_id.timestamp_ms() as i64).unwrap(),
        }
    }
}

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
