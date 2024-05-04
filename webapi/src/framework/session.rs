use axum::{async_trait, extract, http::request::Parts};
use axum_extra::extract::cookie::{Cookie, SameSite};
use reqwest::StatusCode;
use time::Duration;
use ulid::Ulid;

use crate::settings::{SESSION_EXPIRATION_HOURS, SESSION_ID_KEY};

use super::system::{AuthenticatedUser, Role};

/// セッション
#[derive(Clone, Debug)]
pub struct Session {
    pub session_id: Ulid,
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
            session_id: Ulid::from_string(str).unwrap_or(Ulid::new()),
            user: AuthenticatedUser {
                id: "xxx".to_string(),
                roles: vec![Role::General],
                name: "takuya".to_string(),
            },
        });
    }
    None
}


pub fn mk_cookie(session_id: String) -> Cookie<'static> {
    let mut c = Cookie::new(SESSION_ID_KEY, session_id);
    c.set_max_age(Duration::hours(SESSION_EXPIRATION_HOURS));
    c.set_secure(true);
    c.set_http_only(true);
    c.set_path("/");
    c.set_same_site(SameSite::Lax);

    c
}