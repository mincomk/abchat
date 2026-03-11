use pubsub_rs::Pubsub;

use crate::{Message, chat::ChatSession};

#[derive(Clone)]
pub struct ChatManager {
    pubsub: Pubsub<String, Message>,
}

impl ChatManager {
    pub fn new() -> Self {
        Self {
            pubsub: Pubsub::new(),
        }
    }

    pub async fn subscribe(&self, channel_id: &str, echo: bool) -> ChatSession {
        ChatSession::new(channel_id, echo, self.pubsub.clone()).await
    }
}
