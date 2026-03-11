use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

use crate::{AppResult, AppState, User, auth::AuthError, util};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // username
    pub iat: usize,
    pub exp: usize,
    pub is_admin: bool,
}

pub fn sign_token(secret: &[u8], username: &str, is_admin: bool) -> AppResult<String> {
    let now = util::now_u64() / 1000;
    let claims = Claims {
        sub: username.to_string(),
        iat: now as usize,
        exp: (now + 24 * 3600) as usize,
        is_admin,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret),
    )
    .map_err(|_| AuthError::InvalidToken.into())
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

pub fn auth_claims(secret: &[u8], token: &str) -> AppResult<Claims> {
    verify_token(secret, token)
}
