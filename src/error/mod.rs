use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

use crate::domain::MoneyError;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("authentication failed")]
    Unauthorized,

    #[error("forbidden")]
    Forbidden,

    #[error("not found: {0}")]
    NotFound(String),

    #[error("validation error: {0}")]
    Validation(String),

    #[error("insufficient funds")]
    InsufficientFunds,

    #[error("currency mismatch")]
    CurrencyMismatch,

    #[error("conflict: {0}")]
    Conflict(String),

    #[error(transparent)]
    Money(#[from] MoneyError),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),

    #[error("internal error")]
    Internal,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
            AppError::Forbidden => (StatusCode::FORBIDDEN, self.to_string()),
            AppError::NotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::Validation(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::InsufficientFunds => (StatusCode::UNPROCESSABLE_ENTITY, self.to_string()),
            AppError::CurrencyMismatch => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::Conflict(_) => (StatusCode::CONFLICT, self.to_string()),
            AppError::Money(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::Sqlx(e) => {
                tracing::error!(error = %e, "database error");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal server error".into())
            }
            AppError::Anyhow(e) => {
                tracing::error!(error = %e, "internal error");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal server error".into())
            }
            AppError::Internal => (StatusCode::INTERNAL_SERVER_ERROR, "internal server error".into()),
        };

        let body = Json(json!({
            "error": message,
        }));

        (status, body).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
