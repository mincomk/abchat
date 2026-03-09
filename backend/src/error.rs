use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

use crate::{ErrorResposne, auth::AuthError};

#[derive(Debug, Clone, Error)]
pub enum UserError {
    #[error("Cannot delete yourself")]
    CannotDeleteYourself,

    #[error("User not found")]
    UserNotFound,
}

#[derive(Debug, Clone, Error)]
pub enum ServiceError {
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
}

#[derive(Debug, Clone, Error)]
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
