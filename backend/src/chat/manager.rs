use std::sync::Arc;

use dashmap::DashMap;
use pubsub_rs::Pubsub;
use tokio::task::JoinHandle;

use crate::{
    Message,
    chat::{ChatSession, PersistenceBroker, PubSubBroker},
    persistence::Persistence,
    pubsub::MessagePubSub,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BrokerType {
    Persistence,
    PubSub,
}

#[derive(Clone)]
pub struct ChatManager {
    pubsub: Pubsub<String, Message>,
    brokers: Arc<DashMap<(String, BrokerType), JoinHandle<()>>>,
    persistence: Arc<dyn Persistence>,
    external_pubsub: Arc<dyn MessagePubSub>,
}

impl ChatManager {
    pub fn new(persistence: Arc<dyn Persistence>, external_pubsub: Arc<dyn MessagePubSub>) -> Self {
        Self {
            pubsub: Pubsub::new(),
            brokers: Arc::new(DashMap::new()),
            persistence,
            external_pubsub,
        }
    }

    pub async fn subscribe(&self, channel_id: &str, echo: bool) -> ChatSession {
        let session = ChatSession::new(channel_id, echo, self.pubsub.clone()).await;

        self.ensure_persistence_broker(channel_id).await;
        self.ensure_pubsub_broker(channel_id).await;

        session
    }

    async fn ensure_persistence_broker(&self, channel_id: &str) {
        let key = (channel_id.to_string(), BrokerType::Persistence);
        if let dashmap::mapref::entry::Entry::Vacant(e) = self.brokers.entry(key.clone()) {
            let channel_id = channel_id.to_string();
            let persistence = self.persistence.clone();
            let pubsub_rs = self.pubsub.clone();
            let brokers = self.brokers.clone();
            let key_clone = key.clone();

            let handle = tokio::spawn(async move {
                let session = ChatSession::new(&channel_id, false, pubsub_rs).await;
                let broker = PersistenceBroker::new(session, persistence);
                broker.run().await;
                brokers.remove(&key_clone);
            });

            e.insert(handle);
        }
    }

    async fn ensure_pubsub_broker(&self, channel_id: &str) {
        let key = (channel_id.to_string(), BrokerType::PubSub);
        if let dashmap::mapref::entry::Entry::Vacant(e) = self.brokers.entry(key.clone()) {
            let channel_id = channel_id.to_string();
            let external_pubsub = self.external_pubsub.clone();
            let pubsub_rs = self.pubsub.clone();
            let brokers = self.brokers.clone();
            let key_clone = key.clone();

            let handle = tokio::spawn(async move {
                let session = ChatSession::new(&channel_id, false, pubsub_rs).await;
                let broker = PubSubBroker::new(session, external_pubsub);
                broker.run().await;
                brokers.remove(&key_clone);
            });

            e.insert(handle);
        }
    }
}
