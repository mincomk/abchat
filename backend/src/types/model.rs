use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct User {
    pub username: String,
    pub nickname: String,
    pub is_admin: bool,
    #[serde(skip_serializing)]
    pub password_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Message {
    pub id: String,
    pub content: String,
    pub timestamp: u64,
    pub channel_id: String,
}
