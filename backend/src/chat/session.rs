use std::sync::Arc;

use dashmap::DashSet;
use pubsub_rs::{Pubsub, PubsubError, Subscriber};

use crate::{AppError, AppResult, Message, ServiceError};

#[derive(Clone)]
pub struct ChatSession {
    pub channel_id: String,
    pub echo: bool,

    sent_ids: Arc<DashSet<String>>,

    subscriber: Subscriber<String, Message>,
    pubsub: Pubsub<String, Message>,
}

impl ChatSession {
    pub async fn new(channel_id: &str, echo: bool, pubsub: Pubsub<String, Message>) -> Self {
        Self {
            channel_id: channel_id.to_string(),
            echo,
            sent_ids: Default::default(),
            subscriber: pubsub.subscribe(vec![make_topic(channel_id)]).await,
            pubsub,
        }
    }

    pub async fn send(&self, message: Message) {
        if !self.echo {
            self.sent_ids.insert(message.id.clone());
        }

        self.pubsub
            .publish(make_topic(&self.channel_id), message)
            .await;
    }

    pub async fn recv(&self) -> Result<Message, ServiceError> {
        if self.echo {
            let (_, msg) = self.subscriber.recv().await?;
            Ok(msg)
        } else {
            let msg = loop {
                let (_, msg) = self.subscriber.recv().await?;

                // skip until sent_ids don't have the id
                let got_id = self.sent_ids.remove(&msg.id);
                if got_id.is_none() {
                    break msg;
                }
            };

            Ok(msg)
        }
    }
}

fn make_topic(channel_id: &str) -> String {
    format!("chat:{}", channel_id)
}
