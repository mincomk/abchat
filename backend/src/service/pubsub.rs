use async_trait::async_trait;

use crate::{Message, ServiceError};

pub mod in_memory;
pub mod redis;

pub use in_memory::InMemoryPubSub;

#[async_trait]
pub trait MessageSubscriber: Send + Sync {
    async fn next(&self) -> Result<Message, ServiceError>;
}

#[async_trait]
pub trait MessagePubSub: Send + Sync {
    async fn publish(&self, topic: &str, message: Message) -> Result<(), ServiceError>;
    async fn subscribe(&self, topic: &str) -> Result<Box<dyn MessageSubscriber>, ServiceError>;
}
