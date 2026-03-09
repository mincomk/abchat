use async_trait::async_trait;
use redis::{Client, AsyncCommands};
use crate::{AppResult, Message, ServiceError, AppError};
use super::{MessagePubSub, MessageSubscriber};
use tokio::sync::Mutex;
use futures_util::StreamExt;

pub struct RedisMessagePubSub {
    client: Client,
}

impl RedisMessagePubSub {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl MessagePubSub for RedisMessagePubSub {
    async fn publish(&self, topic: &str, message: Message) -> AppResult<()> {
        let mut conn = self.client.get_multiplexed_async_connection().await
            .map_err(|e| AppError::Service(ServiceError::Redis(e)))?;
        
        let payload = serde_json::to_string(&message)
            .map_err(|e| AppError::Service(ServiceError::Json(e)))?;

        conn.publish::<&str, String, ()>(topic, payload).await
            .map_err(|e| AppError::Service(ServiceError::Redis(e)))?;
        
        Ok(())
    }

    async fn subscribe(&self, topic: &str) -> Box<dyn MessageSubscriber> {
        // Note: get_async_pubsub might require &mut client depending on version, 
        // but looking at grep it was pub async fn get_async_pubsub(&self)
        let pubsub = self.client.get_async_pubsub().await.expect("Failed to get pubsub connection");
        let mut pubsub = pubsub;
        pubsub.subscribe(topic).await.expect("Failed to subscribe");
        
        Box::new(RedisMessageSubscriber {
            pubsub: Mutex::new(pubsub),
        })
    }
}

pub struct RedisMessageSubscriber {
    pubsub: Mutex<redis::aio::PubSub>,
}

#[async_trait]
impl MessageSubscriber for RedisMessageSubscriber {
    async fn next(&self) -> AppResult<Message> {
        let mut pubsub = self.pubsub.lock().await;
        let mut stream = pubsub.on_message();
        if let Some(msg) = stream.next().await {
            let payload: String = msg.get_payload().map_err(|e| AppError::Service(ServiceError::Redis(e)))?;
            let message: Message = serde_json::from_str(&payload)
                .map_err(|e| AppError::Service(ServiceError::Json(e)))?;
            Ok(message)
        } else {
            Err(AppError::Service(ServiceError::Redis(redis::RedisError::from((
                redis::ErrorKind::Client,
                "PubSub stream closed",
            )))))
        }
    }
}
