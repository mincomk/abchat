use dbridge_core::{Channel, Message, Platform, PlatformEvent, User};
use dbridge_error::DBridgeError;
use persistence::Persistence;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

pub struct BridgeEngine {
    platforms: Arc<RwLock<HashMap<String, Arc<dyn Platform>>>>,
    channels: Arc<RwLock<Vec<Channel>>>,
    persistence: Arc<dyn Persistence>,
}

impl BridgeEngine {
    pub fn new(persistence: Arc<dyn Persistence>) -> Self {
        Self {
            platforms: Arc::new(RwLock::new(HashMap::new())),
            channels: Arc::new(RwLock::new(Vec::new())),
            persistence,
        }
    }

    pub async fn add_platform(&self, platform: Arc<dyn Platform>) {
        let mut platforms = self.platforms.write().await;
        platforms.insert(platform.name(), platform);
    }

    pub async fn add_channel(&self, channel: Channel) {
        let mut channels = self.channels.write().await;
        channels.push(channel);
    }

    pub async fn run(&self) -> Result<(), DBridgeError> {
        let (tx, mut rx) = mpsc::channel(100);

        let platforms = self.platforms.read().await;

        for platform in platforms.values() {
            let platform = platform.clone();
            let tx = tx.clone();
            tokio::spawn(async move {
                if let Err(e) = platform.listen(tx).await {
                    error!("Platform {} error: {}", platform.name(), e);
                }
            });
        }
        drop(platforms);

        info!("Bridge engine running...");

        while let Some(event) = rx.recv().await {
            match event {
                PlatformEvent::MessageReceived {
                    platform,
                    remote_channel_id,
                    message,
                } => {
                    self.handle_message(platform, remote_channel_id, message)
                        .await;
                }
                PlatformEvent::MemberJoined {
                    platform,
                    remote_channel_id,
                    identifier,
                    user,
                } => {
                    self.handle_member_event(platform, remote_channel_id, identifier, user, true)
                        .await;
                }
                PlatformEvent::MemberLeft {
                    platform,
                    remote_channel_id,
                    identifier,
                    user,
                } => {
                    self.handle_member_event(platform, remote_channel_id, identifier, user, false)
                        .await;
                }
            }
        }

        Ok(())
    }

    async fn handle_member_event(
        &self,
        platform_name: String,
        remote_channel_id: String,
        _identifier: String,
        user: User,
        joined: bool,
    ) {
        let action = if joined { "joined" } else { "left" };
        let content = format!("**System**: {} {} the channel.", user.nickname, action);

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let message = Message {
            id: uuid::Uuid::new_v4().to_string(),
            sender: User {
                username: "system".into(),
                nickname: "System".into(),
                is_admin: true,
            },
            content,
            timestamp,
        };

        self.route_message(platform_name, remote_channel_id, message)
            .await;
    }

    async fn handle_message(&self, platform: String, remote_channel_id: String, message: Message) {
        self.route_message(platform, remote_channel_id, message).await;
    }

    async fn route_message(
        &self,
        source_platform: String,
        source_remote_id: String,
        message: Message,
    ) {
        let platforms = self.platforms.read().await;
        let channels = self.channels.read().await;

        for channel in channels.iter() {
            let mut providers = vec![channel.master_provider.clone()];
            providers.extend(channel.secondary_providers.clone());

            let is_from_this_channel = providers
                .iter()
                .any(|p| p.platform == source_platform && p.remote_id == source_remote_id);

            if is_from_this_channel {
                let mut unique_targets = HashMap::new();
                for p in &providers {
                    unique_targets.insert((p.platform.clone(), p.remote_id.clone()), p.clone());
                }

                for target in unique_targets.values() {
                    if let Err(e) = self
                        .persistence
                        .save_message(&target.platform, &target.remote_id, message.clone())
                        .await
                    {
                        warn!(
                            "Failed to persist message for {}/{}: {}",
                            target.platform, target.remote_id, e
                        );
                    }

                    let is_source =
                        target.platform == source_platform && target.remote_id == source_remote_id;

                    let should_forward = if is_source {
                        if let Some(p) = platforms.get(&target.platform) {
                            p.should_echo()
                        } else {
                            false
                        }
                    } else {
                        true
                    };

                    if should_forward {
                        if let Some(platform) = platforms.get(&target.platform) {
                            info!(
                                "[Relay] {} -> {} ({}): {}",
                                platform.name(),
                                target.platform,
                                target.remote_id,
                                message.content
                            );
                            if let Err(e) = platform.send_message(&target.remote_id, &message).await
                            {
                                warn!(
                                    "Failed to send message to platform {}: {:?}",
                                    platform.name(),
                                    e
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}
