use axum::{async_trait, extract, http::request::Parts};
use reqwest::StatusCode;

use super::system::{AuthenticatedUser, Role};

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
            .get::<Session>()
            .ok_or(StatusCode::UNAUTHORIZED)
            .cloned()
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
