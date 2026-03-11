use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::MessageUser;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct User {
    pub username: String,
    pub nickname: String,
    pub is_admin: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Message {
    pub id: String,
    pub content: String,
    pub timestamp: u64,
    pub channel_id: String,
    pub sender: MessageUser,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub enum NotificationMode {
    All,
    Critical,
    Off,
}

impl ToString for NotificationMode {
    fn to_string(&self) -> String {
        match self {
            NotificationMode::All => "All".to_string(),
            NotificationMode::Critical => "Critical".to_string(),
            NotificationMode::Off => "Off".to_string(),
        }
    }
}

impl From<String> for NotificationMode {
    fn from(s: String) -> Self {
        match s.as_str() {
            "All" => NotificationMode::All,
            "Critical" => NotificationMode::Critical,
            "Off" => NotificationMode::Off,
            _ => NotificationMode::All,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserSettings {
    pub notification_mode: NotificationMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Subscription {
    pub username: String,
    pub endpoint: String,
    pub p256dh: String,
    pub auth: String,
}
