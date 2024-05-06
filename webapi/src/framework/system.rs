use std::{error::Error, fmt::Debug};

use super::logger::{Logger, LoggerInterface};
use axum::response::{IntoResponse, Response};
use reqwest::StatusCode;

pub struct Panic(pub String);

#[derive(Debug)]
pub enum AppError {
    /// 業務上xxxなはずだからunwrapする時に使う
    Unexpected(Logger, String),
    AuthenticationError,
    AutorizationError(String),
    /// ワークフローの最中に発生したエラー
    WorkflowException(StatusCode, String),
}

pub trait IntoAppError: Sized {
    fn into_app_error(self, l: Logger) -> AppError;
}

impl IntoAppError for Panic {
    fn into_app_error(self, l: Logger) -> AppError {
        AppError::Unexpected(l, self.0)
    }
}

pub trait DomainError: IntoAppError {
    fn from_panic(e: Panic) -> Self
    where
        Self: Sized;
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::AuthenticationError => {
                (StatusCode::UNAUTHORIZED, "認証エラー").into_response()
            }
            AppError::AutorizationError(msg) => (StatusCode::FORBIDDEN, msg).into_response(),
            AppError::WorkflowException(code, msg) => (code, msg).into_response(),

            AppError::Unexpected(l, msg) => {
                l.danger(&msg);

                (StatusCode::INTERNAL_SERVER_ERROR, "内部エラー").into_response()
            }
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
