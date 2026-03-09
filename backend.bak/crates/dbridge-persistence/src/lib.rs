use async_trait::async_trait;
use dbridge_core::{Channel, Message};
use dbridge_error::DBridgeError;
use serde::{Deserialize, Serialize};

#[async_trait]
pub trait Persistence: MessageStore + AccountStore + ChannelConfigStore {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub username: String,
    pub password_hash: String,
    pub nickname: String,
    pub is_admin: bool,
}

#[async_trait]
pub trait MessageStore: Send + Sync {
    async fn save_message(
        &self,
        platform: &str,
        channel: &str,
        message: Message,
    ) -> Result<(), DBridgeError>;
    async fn get_messages(
        &self,
        platform: &str,
        channel: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Message>, DBridgeError>;
    async fn get_all_messages(
        &self,
        platform: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<(String, Message)>, DBridgeError>;
}

#[async_trait]
pub trait AccountStore: Send + Sync {
    async fn save_account(&self, account: Account) -> Result<(), DBridgeError>;
    async fn get_account(&self, username: &str) -> Result<Option<Account>, DBridgeError>;
    async fn list_accounts(&self) -> Result<Vec<Account>, DBridgeError>;
    async fn delete_account(&self, username: &str) -> Result<(), DBridgeError>;
}

#[async_trait]
pub trait ChannelConfigStore: Send + Sync {
    async fn save_channel(&self, channel: Channel) -> Result<(), DBridgeError>;
    async fn get_channels(&self) -> Result<Vec<Channel>, DBridgeError>;
}
