use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};

use crate::{AppResult, AppState, User, UserError, auth::AdminUser};

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
    state.persistence.list_users().await.map(Json)
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
    AdminUser(user): AdminUser,
    Path(username): Path<String>,
) -> AppResult<StatusCode> {
    if user.username == username {
        return Err(UserError::CannotDeleteYourself.into());
    }

    let user_to_delete = state.persistence.get_user(&username).await?;
    if user_to_delete.is_none() {
        return Err(UserError::UserNotFound.into());
    }

    state.persistence.delete_user(&username).await?;

    Ok(StatusCode::NO_CONTENT)
}
