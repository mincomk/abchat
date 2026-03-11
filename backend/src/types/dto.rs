use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::User;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LoginResponse {
    pub token: String,
    pub user: User,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AdminChangePasswordRequest {
    pub new_password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateUserAdminRequest {
    pub is_admin: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateUser {
    pub username: String,
    pub password: String,
    pub nickname: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MessageUser {
    pub username: String,
    pub nickname: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ErrorResponse {
    pub message: String,
}
