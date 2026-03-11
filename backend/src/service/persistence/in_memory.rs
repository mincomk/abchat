use async_trait::async_trait;
use dashmap::DashMap;
use crate::{AppResult, Message, User};
use super::Persistence;

pub struct InMemoryPersistence {
    users: DashMap<String, User>,
    passwords: DashMap<String, String>,
    messages: DashMap<String, Vec<Message>>,
}

impl InMemoryPersistence {
    pub fn new() -> Self {
        Self {
            users: DashMap::new(),
            passwords: DashMap::new(),
            messages: DashMap::new(),
        }
    }
}

#[async_trait]
impl Persistence for InMemoryPersistence {
    async fn save_user(&self, u: User) -> AppResult<()> {
        self.users.insert(u.username.clone(), u);
        Ok(())
    }

    async fn list_users(&self) -> AppResult<Vec<User>> {
        Ok(self.users.iter().map(|kv| kv.value().clone()).collect())
    }

    async fn get_user(&self, username: &str) -> AppResult<Option<User>> {
        Ok(self.users.get(username).map(|u| kv_to_user(&u)))
    }

    async fn delete_user(&self, username: &str) -> AppResult<()> {
        self.users.remove(username);
        self.passwords.remove(username);
        Ok(())
    }

    async fn get_password_hash(&self, username: &str) -> AppResult<Option<String>> {
        Ok(self.passwords.get(username).map(|p| p.value().clone()))
    }

    async fn set_password_hash(&self, username: &str, hash: &str) -> AppResult<()> {
        self.passwords.insert(username.to_string(), hash.to_string());
        Ok(())
    }

    async fn add_message(&self, message: Message) -> AppResult<()> {
        self.messages
            .entry(message.channel_id.clone())
            .or_default()
            .push(message);
        Ok(())
    }

    async fn list_messages(
        &self,
        channel_id: String,
        limit: u32,
        offset: u32,
    ) -> AppResult<Vec<Message>> {
        if let Some(msgs) = self.messages.get(&channel_id) {
            let limit = limit as usize;
            let offset = offset as usize;
            let start = offset.min(msgs.len());
            let end = (start + limit).min(msgs.len());
            Ok(msgs[start..end].to_vec())
        } else {
            Ok(vec![])
        }
    }
}

fn kv_to_user(u: &User) -> User {
    u.clone()
}
