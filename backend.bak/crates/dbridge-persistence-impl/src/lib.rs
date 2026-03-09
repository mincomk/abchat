use async_trait::async_trait;
use dbridge_core::{Channel, Message};
use dbridge_error::DBridgeError;
use dbridge_persistence::Persistence;
pub use dbridge_persistence::{Account, AccountStore, ChannelConfigStore, MessageStore};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct InMemoryPersistence {
    /// platform -> channel -> messages
    storage: Arc<RwLock<HashMap<String, HashMap<String, Vec<Message>>>>>,
    /// username -> account
    accounts: Arc<RwLock<HashMap<String, Account>>>,
    /// channel_id -> channel
    channels: Arc<RwLock<HashMap<String, Channel>>>,
}

impl Persistence for InMemoryPersistence {}

impl InMemoryPersistence {
    pub fn new() -> Self {
        Self {
            storage: Arc::new(RwLock::new(HashMap::new())),
            accounts: Arc::new(RwLock::new(HashMap::new())),
            channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl MessageStore for InMemoryPersistence {
    async fn save_message(
        &self,
        platform: &str,
        channel: &str,
        message: Message,
    ) -> Result<(), DBridgeError> {
        let mut storage = self.storage.write().await;
        let platform_storage = storage.entry(platform.to_string()).or_default();
        let channel_storage = platform_storage.entry(channel.to_string()).or_default();

        if channel_storage.iter().any(|m| m.id == message.id) {
            return Ok(());
        }

        if channel_storage.len() >= 1000 {
            channel_storage.remove(0);
        }
        channel_storage.push(message);
        Ok(())
    }

    async fn get_messages(
        &self,
        platform: &str,
        channel: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Message>, DBridgeError> {
        let storage = self.storage.read().await;
        let messages = storage
            .get(platform)
            .and_then(|p| p.get(channel))
            .map(|m| m.iter().rev().skip(offset).take(limit).cloned().collect())
            .unwrap_or_default();
        Ok(messages)
    }

    async fn get_all_messages(
        &self,
        platform: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<(String, Message)>, DBridgeError> {
        let storage = self.storage.read().await;
        let mut all_messages = Vec::new();

        if let Some(platform_storage) = storage.get(platform) {
            for (channel_id, messages) in platform_storage {
                for msg in messages {
                    all_messages.push((channel_id.clone(), msg.clone()));
                }
            }
        }

        all_messages.sort_by(|a, b| b.1.timestamp.cmp(&a.1.timestamp));
        let paginated = all_messages.into_iter().skip(offset).take(limit).collect();

        Ok(paginated)
    }
}

#[async_trait]
impl AccountStore for InMemoryPersistence {
    async fn save_account(&self, account: Account) -> Result<(), DBridgeError> {
        let mut accounts = self.accounts.write().await;
        accounts.insert(account.username.clone(), account);
        Ok(())
    }

    async fn get_account(&self, username: &str) -> Result<Option<Account>, DBridgeError> {
        let accounts = self.accounts.read().await;
        Ok(accounts.get(username).cloned())
    }

    async fn list_accounts(&self) -> Result<Vec<Account>, DBridgeError> {
        let accounts = self.accounts.read().await;
        Ok(accounts.values().cloned().collect())
    }

    async fn delete_account(&self, username: &str) -> Result<(), DBridgeError> {
        let mut accounts = self.accounts.write().await;
        accounts.remove(username);
        Ok(())
    }
}

#[async_trait]
impl ChannelConfigStore for InMemoryPersistence {
    async fn save_channel(&self, channel: Channel) -> Result<(), DBridgeError> {
        let mut channels = self.channels.write().await;
        channels.insert(channel.id.clone(), channel);
        Ok(())
    }

    async fn get_channels(&self) -> Result<Vec<Channel>, DBridgeError> {
        let channels = self.channels.read().await;
        Ok(channels.values().cloned().collect())
    }
}
