use std::sync::Arc;

use dashmap::DashSet;
use tracing::{debug, error};

use crate::{
    NotificationMode,
    chat::{ChatSession, make_topic},
    persistence::Persistence,
    pubsub::MessagePubSub,
    service::notifications::NotificationService,
};
use regex::Regex;

pub struct PersistenceBroker {
    session: ChatSession,
    persistence: Arc<dyn Persistence>,
}

impl PersistenceBroker {
    pub fn new(session: ChatSession, persistence: Arc<dyn Persistence>) -> Self {
        Self {
            session,
            persistence,
        }
    }

    pub async fn run(self) {
        debug!(
            "Persistence broker started for channel: {}",
            self.session.channel_id
        );
        loop {
            match self.session.recv().await {
                Ok(msg) => {
                    if let Err(e) = self.persistence.add_message(msg).await {
                        // We don't break here, just log.
                        // It might be a duplicate message from another node,
                        // which is fine if we have unique constraints.
                        debug!("Failed to persist message (might be duplicate): {:?}", e);
                    }
                }
                Err(e) => {
                    error!("Persistence broker session receive error: {:?}", e);
                    break;
                }
            }
        }
        debug!(
            "Persistence broker stopped for channel: {}",
            self.session.channel_id
        );
    }
}

pub struct PubSubBroker {
    session: ChatSession,
    pubsub: Arc<dyn MessagePubSub>,
    published_ids: Arc<DashSet<String>>,
}

impl PubSubBroker {
    pub fn new(session: ChatSession, pubsub: Arc<dyn MessagePubSub>) -> Self {
        Self {
            session,
            pubsub,
            published_ids: Arc::new(DashSet::new()),
        }
    }

    pub async fn run(self) {
        let topic = make_topic(&self.session.channel_id);
        debug!(
            "PubSub broker started for channel: {} (topic: {})",
            self.session.channel_id, topic
        );

        let external_sub = match self.pubsub.subscribe(&topic).await {
            Ok(sub) => sub,
            Err(e) => {
                error!("Failed to subscribe to external PubSub: {:?}", e);
                return;
            }
        };

        let session_fwd = self.session.clone();
        let pubsub_fwd = self.pubsub.clone();
        let topic_fwd = topic.clone();
        let published_ids_fwd = self.published_ids.clone();

        // Task 1: Local Session -> External PubSub
        let forward_task = tokio::spawn(async move {
            loop {
                match session_fwd.recv().await {
                    Ok(msg) => {
                        // If we received a message from local, publish it to external
                        // and track its ID to avoid processing it when it comes back.
                        published_ids_fwd.insert(msg.id.clone());

                        // Limit cache size simple way
                        if published_ids_fwd.len() > 1000 {
                            // This is very naive, but helps avoid memory leak for this task
                            published_ids_fwd.clear();
                        }

                        if let Err(e) = pubsub_fwd.publish(&topic_fwd, msg).await {
                            error!("Failed to publish to external PubSub: {:?}", e);
                        }
                    }
                    Err(e) => {
                        error!("PubSub broker local receive error: {:?}", e);
                        break;
                    }
                }
            }
        });

        // Task 2: External PubSub -> Local Session
        let session_recv = self.session.clone();
        let published_ids_recv = self.published_ids.clone();
        let receive_task = tokio::spawn(async move {
            loop {
                match external_sub.next().await {
                    Ok(msg) => {
                        // If we received a message from external, check if we were the one who published it.
                        if published_ids_recv.remove(&msg.id).is_some() {
                            // Already handled, ignore it.
                            continue;
                        }

                        // Send to local session.
                        // This will distribute it to all local clients.
                        session_recv.send(msg).await;
                    }
                    Err(e) => {
                        error!("External PubSub receive error: {:?}", e);
                        break;
                    }
                }
            }
        });

        tokio::select! {
            _ = forward_task => {},
            _ = receive_task => {},
        }

        debug!(
            "PubSub broker stopped for channel: {}",
            self.session.channel_id
        );
    }
}

pub struct NotificationBroker {
    session: ChatSession,
    notification_service: Arc<NotificationService>,
}

impl NotificationBroker {
    pub fn new(session: ChatSession, notification_service: Arc<NotificationService>) -> Self {
        Self {
            session,
            notification_service,
        }
    }

    pub async fn run(self) {
        debug!(
            "Notification broker started for channel: {}",
            self.session.channel_id
        );

        let mention_re = Regex::new(r"@\(([^)]+)\)").unwrap();

        loop {
            match self.session.recv().await {
                Ok(msg) => {
                    let mut target_modes = vec![NotificationMode::All];
                    let mut targeted_usernames = Vec::new();
                    let mut is_everyone = false;

                    for cap in mention_re.captures_iter(&msg.content) {
                        let mention = &cap[1];
                        if mention == "everyone" {
                            is_everyone = true;
                        } else {
                            targeted_usernames.push(mention.to_string());
                        }
                    }

                    if is_everyone {
                        target_modes.push(NotificationMode::Critical);
                    }

                    let title = format!("{}", msg.sender.nickname);
                    let body = msg.content.clone();

                    if let Err(e) = self
                        .notification_service
                        .send_targeted_notification(
                            &title,
                            &body,
                            target_modes,
                            targeted_usernames,
                            Some(msg.sender.username),
                        )
                        .await
                    {
                        error!("Notification broker failed to send notification: {:?}", e);
                    }
                }
                Err(e) => {
                    error!("Notification broker session receive error: {:?}", e);
                    break;
                }
            }
        }

        debug!(
            "Notification broker stopped for channel: {}",
            self.session.channel_id
        );
    }
}
