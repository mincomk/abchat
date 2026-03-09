use dbridge_error::DBridgeError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub username: String,
    pub nickname: String,
    pub is_admin: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub sender: User,
    pub content: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provider {
    pub platform: String,
    pub remote_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub id: String,
    pub name: String,
    pub master_provider: Provider,
    pub secondary_providers: Vec<Provider>,
}

impl Channel {
    pub fn new(id: String, name: String, master: Provider) -> Self {
        Self {
            id,
            name,
            master_provider: master,
            secondary_providers: Vec::new(),
        }
    }

    pub fn add_secondary_provider(&mut self, provider: Provider) {
        self.secondary_providers.push(provider);
    }
}

pub enum PlatformEvent {
    MessageReceived {
        platform: String,
        remote_channel_id: String,
        message: Message,
    },
    MemberJoined {
        platform: String,
        remote_channel_id: String,
        identifier: String,
        user: User,
    },
    MemberLeft {
        platform: String,
        remote_channel_id: String,
        identifier: String,
        user: User,
    },
}

#[async_trait::async_trait]
pub trait Platform: Send + Sync {
    fn name(&self) -> String;
    async fn send_message(&self, remote_channel_id: &str, message: &Message) -> Result<String, DBridgeError>;
    async fn get_members(&self, remote_channel_id: &str) -> Result<Vec<User>, DBridgeError>;
    async fn get_messages(&self, remote_channel_id: &str, limit: usize, offset: usize) -> Result<Vec<Message>, DBridgeError>;
    async fn listen(&self, tx: tokio::sync::mpsc::Sender<PlatformEvent>) -> Result<(), DBridgeError>;
    fn should_echo(&self) -> bool;
}
