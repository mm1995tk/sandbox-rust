use std::{error::Error, fmt::Debug};

use super::logger::{Logger, LoggerInterface};
use axum::response::{IntoResponse, Response};
use reqwest::StatusCode;

pub struct Panic<T: Into<String>>(pub T);

#[derive(Debug)]
pub enum AppError<'a> {
    /// 業務上xxxなはずだからunwrapする時に使う
    Unexpected(&'a Logger, String),
    AuthenticationError,
    AutorizationError(String),
    /// ワークフローの最中に発生したエラー
    WorkflowException(StatusCode, String),
}

pub trait IntoAppError: Sized {
    fn into_app_error(self, l: &Logger) -> AppError;
}

impl<T: Into<String>> IntoAppError for Panic<T> {
    fn into_app_error(self, l: &Logger) -> AppError {
        AppError::Unexpected(l, self.0.into())
    }
}

pub trait DomainError: Debug + Error + IntoAppError {
    fn from_panic<T: Into<String>>(e: Panic<T>) -> Self
    where
        Self: Sized;
}

impl<'a> IntoResponse for AppError<'a> {
    fn into_response(self) -> Response {
        match self {
            AppError::AuthenticationError => (StatusCode::UNAUTHORIZED, "認証エラー").into_response(),
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
