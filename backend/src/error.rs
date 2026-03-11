use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use thiserror::Error;

use crate::{auth::AuthError, ErrorResponse};

#[derive(Debug, Error)]
pub enum UserError {
    #[error("Cannot delete yourself")]
    CannotDeleteYourself,

    #[error("User not found")]
    UserNotFound,

    #[error("User already exists")]
    UserAlreadyExists,

    #[error("Message validation failure: {0}")]
    MessageValidationFailed(String),
}

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("Serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("AuthError: {0}")]
    Auth(#[from] AuthError),

    #[error("Internal ServiceError: {0}")]
    Service(#[from] ServiceError),

    #[error("UserError: {0}")]
    UserError(#[from] UserError),
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::Service(ServiceError::Json(e))
    }
}

pub type AppResult<T> = Result<T, AppError>;

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let r = match self {
            AppError::Auth(e) => (StatusCode::UNAUTHORIZED, err(e)),
            AppError::Service(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                err("Internal server error"),
            ),
            AppError::UserError(e) => match e {
                UserError::CannotDeleteYourself => (StatusCode::FORBIDDEN, err(e)),
                UserError::UserNotFound => (StatusCode::NOT_FOUND, err(e)),
                UserError::UserAlreadyExists => (StatusCode::CONFLICT, err(e)),
                UserError::MessageValidationFailed(_) => (StatusCode::BAD_REQUEST, err(e)),
            },
        };

        r.into_response()
    }
}

fn err(s: impl ToString) -> Json<ErrorResponse> {
    Json(ErrorResponse {
        message: s.to_string(),
    })
}
