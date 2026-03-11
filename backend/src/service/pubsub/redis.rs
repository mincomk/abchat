use super::{MessagePubSub, MessageSubscriber};
use crate::{Message, ServiceError};
use async_trait::async_trait;
use futures_util::StreamExt;
use redis::{AsyncCommands, Client};
use tokio::sync::Mutex;

pub struct RedisMessagePubSub {
    client: Client,
}

impl RedisMessagePubSub {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn connect(url: &str) -> Result<Self, ServiceError> {
        let client = Client::open(url)?;

        Ok(Self::new(client))
    }
}

#[async_trait]
impl MessagePubSub for RedisMessagePubSub {
    async fn publish(&self, topic: &str, message: Message) -> Result<(), ServiceError> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let payload = serde_json::to_string(&message)?;
        conn.publish::<&str, String, ()>(topic, payload).await?;

        Ok(())
    }

    async fn subscribe(&self, topic: &str) -> Result<Box<dyn MessageSubscriber>, ServiceError> {
        let pubsub = self.client.get_async_pubsub().await?;

        let mut pubsub = pubsub;
        pubsub.subscribe(topic).await?;

        Ok(Box::new(RedisMessageSubscriber {
            pubsub: Mutex::new(pubsub),
        }))
    }
}

pub struct RedisMessageSubscriber {
    pubsub: Mutex<redis::aio::PubSub>,
}

#[async_trait]
impl MessageSubscriber for RedisMessageSubscriber {
    async fn next(&self) -> Result<Message, ServiceError> {
        let mut pubsub = self.pubsub.lock().await;
        let mut stream = pubsub.on_message();

        if let Some(msg) = stream.next().await {
            let payload: String = msg.get_payload()?;
            let message: Message = serde_json::from_str(&payload)?;
            Ok(message)
        } else {
            Err(ServiceError::Redis(redis::RedisError::from((
                redis::ErrorKind::Client,
                "PubSub stream closed",
            ))))
        }
    }
}
