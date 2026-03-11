use axum::{Json, extract::State};
use crate::{AppResult, AppState, LoginRequest, LoginResponse, auth::{AuthError, jwt::sign_token, hash::verify_password}};

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
        .await?
        .ok_or(AuthError::InvalidCredentials)?;

    if !verify_password(&user.password_hash, &payload.password)? {
        return Err(AuthError::InvalidCredentials.into());
    }

    let token = sign_token(&state.jwt_secret, &user.username, user.is_admin)?;

    Ok(Json(LoginResponse { token, user }))
}
