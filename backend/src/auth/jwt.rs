use jsonwebtoken::{DecodingKey, Validation, decode};
use serde::{Deserialize, Serialize};

use crate::{AppResult, AppState, User, auth::AuthError};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // username
    pub iat: usize,
    pub exp: usize,
}

pub fn verify_token(secret: &[u8], token: &str) -> AppResult<Claims> {
    let validation = Validation::default();

    let decoded = decode::<Claims>(token, &DecodingKey::from_secret(secret), &validation)
        .map_err(|_| AuthError::InvalidToken)?;

    Ok(decoded.claims)
}

pub async fn auth_user(state: &AppState, token: &str) -> AppResult<User> {
    let claims = verify_token(&state.jwt_secret, token)?;

    let user = state
        .persistence
        .get_user(&claims.sub)
        .await?
        .ok_or(AuthError::AccountNotExist)?;

    Ok(user)
}
