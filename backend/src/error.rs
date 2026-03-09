use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use thiserror::Error;

use crate::{auth::AuthError, ErrorResposne};

#[derive(Debug, Error)]
pub enum UserError {
    #[error("Cannot delete yourself")]
    CannotDeleteYourself,

    #[error("User not found")]
    UserNotFound,
}

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("Serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
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
            },
        };

        r.into_response()
    }
}

fn err(s: impl ToString) -> Json<ErrorResposne> {
    Json(ErrorResposne {
        message: s.to_string(),
    })
}
