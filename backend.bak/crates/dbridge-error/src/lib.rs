use thiserror::Error;

#[derive(Debug, Error)]
pub enum DBridgeError {
    #[error("Persistence error: {0}")]
    Persistence(String),

    #[error("Platform error on {platform}: {message}")]
    Platform { platform: String, message: String },

    #[error("Authorization error: {0}")]
    Auth(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Internal engine error: {0}")]
    Internal(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("NotFound: {0}")]
    NotFound(String),
}

pub type DBridgeResult<T> = Result<T, DBridgeResultError>;

#[derive(Debug, Error)]
pub enum DBridgeResultError {
    #[error(transparent)]
    DBridge(#[from] DBridgeError),

    #[error("Anyhow: {0}")]
    Anyhow(String),
}

#[cfg(feature = "axum")]
mod axum_impl {
    use super::*;
    use axum::http::StatusCode;
    use axum::response::{IntoResponse, Response};
    use serde_json::json;

    impl IntoResponse for DBridgeError {
        fn into_response(self) -> Response {
            let (status, message) = match self {
                DBridgeError::Auth(msg) => (StatusCode::UNAUTHORIZED, msg),
                DBridgeError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
                DBridgeError::Validation(msg) => (StatusCode::BAD_REQUEST, msg),
                _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            };

            let body = axum::Json(json!({ "error": message }));
            (status, body).into_response()
        }
    }
}
