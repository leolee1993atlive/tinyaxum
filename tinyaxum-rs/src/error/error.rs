use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

use crate::{common::vo::response::R, error::localizer};

/// Unified application error type that can represent any error in the system.
///
/// This enum is designed to eventually replace all current error representations
/// (`AuthError`, `SqlParserError`, `String` errors, `bool` failures, `Option::None`
/// failures) across the Rust backend.  New code should prefer `AppError` and the
/// `AppResult<T>` alias.
#[derive(Error, Debug)]
pub enum AppError {
    #[error("JWT错误：{0}")]
    JwtTokenError(String),

    #[error("数据库错误: {0}")]
    DbError(#[from] sea_orm::DbErr),

    #[error("认证错误: {0}")]
    Auth(String),

    #[error("验证错误: {0}")]
    Validation(String),

    #[error("未找到: {0}")]
    NotFound(String),

    #[error("内部错误: {0}")]
    Internal(String),

    #[error("错误请求: {0}")]
    BadRequest(String),
}

impl From<validator::ValidationErrors> for AppError {
    fn from(err: validator::ValidationErrors) -> Self {
        AppError::Validation(err.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Internal(err.to_string())
    }
}

impl From<regex::Error> for AppError {
    fn from(err: regex::Error) -> Self {
        AppError::Internal(err.to_string())
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Internal(err.to_string())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::DbError(e) => {
                tracing::error!(variant = "DbError", error = %e, "应用错误: 数据库错误");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    localizer::translate_db_error(e),
                )
            }
            AppError::JwtTokenError(e) => {
                tracing::warn!(variant = "JwtTokenError", error = %e, "应用错误: JWT错误");
                (StatusCode::UNAUTHORIZED, e.to_string())
            }
            AppError::Auth(e) => {
                tracing::warn!(variant = "Auth", error = %e, "应用错误: 认证错误");
                (StatusCode::UNAUTHORIZED, e.clone())
            }
            AppError::Validation(e) => {
                tracing::warn!(variant = "Validation", error = %e, "应用错误: 验证错误");
                (StatusCode::UNPROCESSABLE_ENTITY, e.clone())
            }
            AppError::NotFound(e) => {
                tracing::warn!(variant = "NotFound", error = %e, "应用错误: 未找到");
                (StatusCode::NOT_FOUND, e.clone())
            }
            AppError::Internal(e) => {
                tracing::error!(variant = "Internal", error = %e, "应用错误: 内部错误");
                (StatusCode::INTERNAL_SERVER_ERROR, e.clone())
            }
            AppError::BadRequest(e) => {
                tracing::warn!(variant = "BadRequest", error = %e, "应用错误: 错误请求");
                (StatusCode::BAD_REQUEST, e.clone())
            }
        };
        let body =
            R::<()>::error_with_code_and_message(status.as_u16(), message);
        (status, body).into_response()
    }
}

/// Convenience alias for results that use the unified `AppError`.
pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use axum::{body::to_bytes, response::IntoResponse};

    use super::*;

    #[tokio::test]
    async fn test_db_error_response_body_is_translated() {
        let db_err = sea_orm::DbErr::Custom("boom".to_string());
        let app_err = AppError::DbError(db_err);
        let response = app_err.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let body_bytes =
            to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
        assert!(
            body_str.contains("boom"),
            "response body should contain the original message, got: {}",
            body_str,
        );
    }
}
