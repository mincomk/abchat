use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};

use crate::{
    AdminChangePasswordRequest, AppResult, AppState, CreateUser, UpdateNicknameRequest, UpdateUserAdminRequest, User,
    UserError,
    auth::{AdminUser, hash::hash_password},
};

#[utoipa::path(
    post,
    path = "/admin/register",
    tag = "admin",
    security(("bearer_auth" = [])),
    request_body = CreateUser,
    responses(
        (status = 201, description = "User registered"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn register_user(
    State(state): State<AppState>,
    _admin: AdminUser,
    Json(payload): Json<CreateUser>,
) -> AppResult<StatusCode> {
    if state
        .persistence
        .get_user(&payload.username)
        .await?
        .is_some()
    {
        return Err(UserError::UserAlreadyExists.into());
    }

    let password_hash = hash_password(&payload.password)?;

    let user = User {
        username: payload.username.clone(),
        nickname: payload.nickname,
        is_admin: false,
    };

    state.persistence.save_user(user).await?;
    state.persistence.set_password_hash(&payload.username, &password_hash).await?;

    Ok(StatusCode::CREATED)
}

#[utoipa::path(
    get,
    path = "/admin/accounts",
    tag = "admin",
    security(("bearer_auth" = [])),
    responses(
        (status = OK, description = "List of users", body = Vec<User>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn list_users(
    State(state): State<AppState>,
    _admin: AdminUser,
) -> AppResult<Json<Vec<User>>> {
    state.persistence.list_users(1000, 0).await.map(Json)
}

#[utoipa::path(
    delete,
    path = "/admin/accounts/{username}",
    tag = "admin",
    security(("bearer_auth" = [])),
    params(("username" = String, Path, description = "Username")),
    responses(
        (status = 204, description = "User deleted"),
        (status = 404, description = "User not found"),
        (status = 400, description = "Cannot delete yourself"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn delete_user(
    State(state): State<AppState>,
    AdminUser(admin_user): AdminUser,
    Path(username): Path<String>,
) -> AppResult<StatusCode> {
    if admin_user.username == username {
        return Err(UserError::CannotDeleteYourself.into());
    }

    let user_to_delete = state.persistence.get_user(&username).await?;
    if user_to_delete.is_none() {
        return Err(UserError::UserNotFound.into());
    }

    state.persistence.delete_user(&username).await?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/admin/accounts/{username}/password",
    tag = "admin",
    security(("bearer_auth" = [])),
    params(("username" = String, Path, description = "Username")),
    request_body = AdminChangePasswordRequest,
    responses(
        (status = 200, description = "Password changed successfully"),
        (status = 404, description = "User not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn admin_change_password(
    State(state): State<AppState>,
    _admin: AdminUser,
    Path(username): Path<String>,
    Json(payload): Json<AdminChangePasswordRequest>,
) -> AppResult<StatusCode> {
    let user = state.persistence.get_user(&username).await?;
    if user.is_none() {
        return Err(UserError::UserNotFound.into());
    }

    let new_hash = hash_password(&payload.new_password)?;
    state.persistence.set_password_hash(&username, &new_hash).await?;

    Ok(StatusCode::OK)
}

#[utoipa::path(
    post,
    path = "/admin/accounts/{username}/nickname",
    tag = "admin",
    security(("bearer_auth" = [])),
    params(("username" = String, Path, description = "Username")),
    request_body = UpdateNicknameRequest,
    responses(
        (status = 200, description = "Nickname changed successfully"),
        (status = 404, description = "User not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn admin_change_nickname(
    State(state): State<AppState>,
    _admin: AdminUser,
    Path(username): Path<String>,
    Json(payload): Json<UpdateNicknameRequest>,
) -> AppResult<StatusCode> {
    let mut user = state
        .persistence
        .get_user(&username)
        .await?
        .ok_or(UserError::UserNotFound)?;

    user.nickname = payload.nickname;
    state.persistence.save_user(user).await?;

    Ok(StatusCode::OK)
}

#[utoipa::path(
    patch,
    path = "/admin/accounts/{username}/admin",
    tag = "admin",
    security(("bearer_auth" = [])),
    params(("username" = String, Path, description = "Username")),
    request_body = UpdateUserAdminRequest,
    responses(
        (status = 200, description = "User admin status updated"),
        (status = 404, description = "User not found"),
        (status = 400, description = "Cannot demote yourself"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn update_user_admin(
    State(state): State<AppState>,
    AdminUser(admin_user): AdminUser,
    Path(username): Path<String>,
    Json(payload): Json<UpdateUserAdminRequest>,
) -> AppResult<StatusCode> {
    if admin_user.username == username && !payload.is_admin {
        return Err(UserError::CannotDemoteSelf.into());
    }

    let mut user = state
        .persistence
        .get_user(&username)
        .await?
        .ok_or(UserError::UserNotFound)?;

    user.is_admin = payload.is_admin;
    state.persistence.save_user(user).await?;

    Ok(StatusCode::OK)
}
