use async_trait::async_trait;

use crate::{AppResult, Message};

pub mod redis;

#[async_trait]
pub trait MessageSubscriber: Send + Sync {
    async fn next(&self) -> AppResult<Message>;
}

#[async_trait]
pub trait MessagePubSub: Send + Sync {
    async fn publish(&self, topic: &str, message: Message) -> AppResult<()>;
    async fn subscribe(&self, topic: &str) -> Box<dyn MessageSubscriber>;
}
