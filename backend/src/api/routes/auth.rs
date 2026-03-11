use crate::{
    AppResult, AppState, LoginRequest, LoginResponse,
    auth::{AuthError, hash::verify_password, jwt::sign_token},
};
use axum::{Json, extract::State};

const DUMMY_HASH: &str = "$argon2id$v=19$m=19456,t=2,p=1$zNT0+g5Sr8sF+9G9DSo0AA$RNSbxYsPd5qttRzezg1HFK4WPLxdf9cH9JlLvENbfXE";

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
    let user = state.persistence.get_user(&payload.username).await?;
    let hash = state.persistence.get_password_hash(&payload.username).await?;

    let hash_str = match &hash {
        Some(h) => h,
        None => DUMMY_HASH,
    };

    if !verify_password(hash_str, &payload.password)? || user.is_none() {
        return Err(AuthError::InvalidCredentials.into());
    }

    let user = user.unwrap();
    let token = sign_token(&state.jwt_secret, &user.username, user.is_admin)?;

    Ok(Json(LoginResponse { token, user }))
}
