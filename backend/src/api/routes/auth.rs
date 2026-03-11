use axum::{Json, extract::State};
use crate::{AppResult, AppState, LoginRequest, LoginResponse, auth::{AuthError, jwt::sign_token, hash::verify_password}};

const DUMMY_HASH: &str = "$argon2id$v=19$m=19456,t=2,p=1$ojqFtNSA2NuI/+ZSF82Dyw$TXAS5A39/5nSEXJyQ4R9DQVJiuNWRcsxZDugl2Uy4fM";

#[utoipa::path(
    post,
    path = "/auth/login",
    tag = "auth",
    request_body = LoginRequest,
    responses(
        (status = OK, description = "Login successful", body = LoginResponse),
        (status = 401, description = "Invalid credentials"),
    )
)]
pub async fn login_handler(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> AppResult<Json<LoginResponse>> {
    let user = state
        .persistence
        .get_user(&payload.username)
        .await?;

    let hash = match &user {
        Some(u) => &u.password_hash,
        None => DUMMY_HASH,
    };

    if !verify_password(hash, &payload.password)? || user.is_none() {
        return Err(AuthError::InvalidCredentials.into());
    }

    let user = user.unwrap();
    let token = sign_token(&state.jwt_secret, &user.username, user.is_admin)?;

    Ok(Json(LoginResponse { token, user }))
}
