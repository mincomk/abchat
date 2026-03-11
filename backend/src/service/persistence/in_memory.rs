use crate::{AppResult, Message, NotificationMode, Subscription, User};
use async_trait::async_trait;
use dashmap::DashMap;
use super::Persistence;

pub struct InMemoryPersistence {
    users: DashMap<String, User>,
    passwords: DashMap<String, String>,
    messages: DashMap<String, Vec<Message>>,
    subscriptions: DashMap<String, Vec<Subscription>>,
    user_settings: DashMap<String, NotificationMode>,
}

impl InMemoryPersistence {
    pub fn new() -> Self {
        Self {
            users: DashMap::new(),
            passwords: DashMap::new(),
            messages: DashMap::new(),
            subscriptions: DashMap::new(),
            user_settings: DashMap::new(),
        }
    }
}

#[async_trait]
impl Persistence for InMemoryPersistence {
    async fn save_user(&self, u: User) -> AppResult<()> {
        self.users.insert(u.username.clone(), u);
        Ok(())
    }

    async fn list_users(&self, limit: u32, offset: u32) -> AppResult<Vec<User>> {
        let mut users: Vec<User> = self.users.iter().map(|kv| kv.value().clone()).collect();
        users.sort_by(|a, b| a.username.cmp(&b.username));

        let start = offset as usize;
        let end = (start + limit as usize).min(users.len());

        if start >= users.len() {
            return Ok(vec![]);
        }

        Ok(users[start..end].to_vec())
    }

    async fn get_user(&self, username: &str) -> AppResult<Option<User>> {
        Ok(self.users.get(username).map(|u| kv_to_user(&u)))
    }

    async fn delete_user(&self, username: &str) -> AppResult<()> {
        self.users.remove(username);
        self.passwords.remove(username);
        self.subscriptions.remove(username);
        self.user_settings.remove(username);
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

    async fn add_subscription(&self, sub: Subscription) -> AppResult<()> {
        let mut user_subs = self.subscriptions.entry(sub.username.clone()).or_default();
        // Remove existing sub with same endpoint if any
        user_subs.retain(|s| s.endpoint != sub.endpoint);
        user_subs.push(sub);
        Ok(())
    }

    async fn list_subscriptions(&self, username: &str) -> AppResult<Vec<Subscription>> {
        Ok(self.subscriptions.get(username).map(|s| s.value().clone()).unwrap_or_default())
    }

    async fn delete_subscription(&self, endpoint: &str) -> AppResult<()> {
        for mut kv in self.subscriptions.iter_mut() {
            kv.value_mut().retain(|s| s.endpoint != endpoint);
        }
        Ok(())
    }

    async fn delete_user_subscriptions(&self, username: &str) -> AppResult<()> {
        self.subscriptions.remove(username);
        Ok(())
    }

    async fn get_user_notification_mode(&self, username: &str) -> AppResult<NotificationMode> {
        Ok(self.user_settings.get(username).map(|s| s.value().clone()).unwrap_or(NotificationMode::All))
    }

    async fn set_user_notification_mode(
        &self,
        username: &str,
        mode: NotificationMode,
    ) -> AppResult<()> {
        self.user_settings.insert(username.to_string(), mode);
        Ok(())
    }

    async fn get_subscriptions_by_mode(
        &self,
        modes: Vec<NotificationMode>,
    ) -> AppResult<Vec<Subscription>> {
        let mut results = Vec::new();

        for kv in self.subscriptions.iter() {
            let username = kv.key();
            let subs = kv.value();

            let user_mode = self
                .user_settings
                .get(username)
                .map(|v| v.value().clone())
                .unwrap_or(NotificationMode::All);

            if modes.contains(&user_mode) {
                results.extend(subs.clone());
            }
        }

        Ok(results)
    }
}

fn kv_to_user(u: &User) -> User {
    u.clone()
}
