use axum::{extract::FromRequestParts, http::request::Parts};

use crate::{
    AppError, AppState, User,
    auth::{AuthError, jwt::auth_user},
};

pub struct AdminUser(pub User);

fn extract_bearer(parts: &Parts) -> Result<&str, AuthError> {
    let header = parts
        .headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AuthError::HeaderRequired)?;

    header
        .strip_prefix("Bearer ")
        .ok_or_else(|| AuthError::AccountNotExist)
}

impl FromRequestParts<AppState> for User {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        auth_user(state, extract_bearer(parts)?).await
    }
}

impl FromRequestParts<AppState> for AdminUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let user = auth_user(state, extract_bearer(parts)?).await?;

        if user.is_admin {
            Ok(AdminUser(user))
        } else {
            Err(AuthError::NoAccess.into())
        }
    }
}
