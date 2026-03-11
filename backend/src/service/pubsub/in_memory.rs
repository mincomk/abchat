use super::{MessagePubSub, MessageSubscriber};
use crate::{Message, ServiceError};
use async_trait::async_trait;
use pubsub_rs::Pubsub;

pub struct InMemoryPubSub {
    pubsub: Pubsub<String, Message>,
}

impl InMemoryPubSub {
    pub fn new() -> Self {
        Self {
            pubsub: Pubsub::new(),
        }
    }
}

impl Default for InMemoryPubSub {
    fn default() -> Self {
        Self::new()
    }
}

pub struct InMemorySubscriber {
    subscriber: pubsub_rs::Subscriber<String, Message>,
}

#[async_trait]
impl MessageSubscriber for InMemorySubscriber {
    async fn next(&self) -> Result<Message, ServiceError> {
        match self.subscriber.recv().await {
            Ok((_topic, message)) => Ok(message),
            Err(_) => Err(ServiceError::Internal("Pubsub closed".to_string())),
        }
    }
}

#[async_trait]
impl MessagePubSub for InMemoryPubSub {
    async fn publish(&self, topic: &str, message: Message) -> Result<(), ServiceError> {
        self.pubsub.publish(topic.to_string(), message).await;
        Ok(())
    }

    async fn subscribe(&self, topic: &str) -> Result<Box<dyn MessageSubscriber>, ServiceError> {
        let subscriber = self.pubsub.subscribe(vec![topic.to_string()]).await;
        Ok(Box::new(InMemorySubscriber { subscriber }))
    }
}
