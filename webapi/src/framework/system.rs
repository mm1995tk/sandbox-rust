use std::error::Error;

use axum::response::{IntoResponse, Response};

#[derive(Debug)]
pub enum AppError {
    Unexpected(Box<dyn Error>),
    Unauthorized(Option<String>),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
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
