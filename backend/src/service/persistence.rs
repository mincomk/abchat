use async_trait::async_trait;

use crate::{AppResult, Message, NotificationMode, Subscription, User};

pub mod postgres;
pub mod in_memory;

pub use in_memory::InMemoryPersistence;

#[async_trait]
pub trait Persistence: Send + Sync {
    async fn save_user(&self, u: User) -> AppResult<()>;
    async fn list_users(&self, limit: u32, offset: u32) -> AppResult<Vec<User>>;
    async fn get_user(&self, username: &str) -> AppResult<Option<User>>;
    async fn delete_user(&self, username: &str) -> AppResult<()>;

    async fn get_password_hash(&self, username: &str) -> AppResult<Option<String>>;
    async fn set_password_hash(&self, username: &str, hash: &str) -> AppResult<()>;

    async fn add_message(&self, message: Message) -> AppResult<()>;
    async fn list_messages(
        &self,
        channel_id: String,
        limit: u32,
        offset: u32,
    ) -> AppResult<Vec<Message>>;

    async fn add_subscription(&self, sub: Subscription) -> AppResult<()>;
    async fn list_subscriptions(&self, username: &str) -> AppResult<Vec<Subscription>>;
    async fn delete_subscription(&self, endpoint: &str) -> AppResult<()>;
    async fn delete_user_subscriptions(&self, username: &str) -> AppResult<()>;

    async fn get_user_notification_mode(&self, username: &str) -> AppResult<NotificationMode>;
    async fn set_user_notification_mode(
        &self,
        username: &str,
        mode: NotificationMode,
    ) -> AppResult<()>;
}
