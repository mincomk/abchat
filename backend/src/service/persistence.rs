use async_trait::async_trait;

use crate::{AppResult, Message, User};

#[async_trait]
pub trait Persistence: Send + Sync {
    async fn save_user(&self, u: User) -> AppResult<()>;
    async fn list_users(&self) -> AppResult<Vec<User>>;
    async fn get_user(&self, username: &str) -> AppResult<Option<User>>;
    async fn delete_user(&self, username: &str) -> AppResult<()>;

    async fn add_message(&self, message: Message) -> AppResult<()>;
    async fn list_messages(
        &self,
        channel_id: String,
        limit: u32,
        offset: u32,
    ) -> AppResult<Vec<Message>>;
}
