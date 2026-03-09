use async_trait::async_trait;
use dbridge_core::{Message, Platform, PlatformEvent, User};
use dbridge_error::DBridgeError;
use dbridge_provider_sdk::EventProducer;
use serenity::all::{Http, Client, GatewayIntents, EventHandler, Context, Ready, Message as SerenityMessage, ChannelId};
use serenity::async_trait as serenity_async_trait;
use tokio::sync::mpsc;
use std::sync::Arc;
use tracing::{error, info};

pub struct DiscordProvider {
    http: Arc<Http>,
    token: String,
}

impl DiscordProvider {
    pub fn new(token: String) -> Self {
        Self {
            http: Arc::new(Http::new(&token)),
            token,
        }
    }
}

#[async_trait]
impl Platform for DiscordProvider {
    fn name(&self) -> String {
        "discord".to_string()
    }

    async fn send_message(
        &self,
        remote_channel_id: &str,
        message: &Message,
    ) -> Result<String, DBridgeError> {
        let channel_id: u64 = remote_channel_id
            .parse()
            .map_err(|_| DBridgeError::Validation("Invalid Discord channel ID".to_string()))?;
        let channel_id = ChannelId::new(channel_id);

        let sanitized_content = message.content
            .replace("@everyone", "@\u{200b}everyone")
            .replace("@here", "@\u{200b}here");

        let content = format!("**{}**: {}", message.sender.nickname, sanitized_content);

        let sent_msg = channel_id.send_message(&self.http, serenity::builder::CreateMessage::new().content(content))
            .await
            .map_err(|e| DBridgeError::Platform {
                platform: "discord".into(),
                message: e.to_string(),
            })?;

        Ok(sent_msg.id.to_string())
    }

    async fn get_members(&self, remote_channel_id: &str) -> Result<Vec<User>, DBridgeError> {
        let channel_id: u64 = remote_channel_id
            .parse()
            .map_err(|_| DBridgeError::Validation("Invalid Discord channel ID".to_string()))?;
        let channel_id = ChannelId::new(channel_id);

        let guild_id = self.http.get_channel(channel_id).await
            .map_err(|e| DBridgeError::Platform { platform: "discord".into(), message: e.to_string() })?
            .guild()
            .ok_or_else(|| DBridgeError::Platform { platform: "discord".into(), message: "Channel is not in a guild".into() })?
            .guild_id;

        let members = guild_id.members(&self.http, None, None).await
            .map_err(|e| DBridgeError::Platform { platform: "discord".into(), message: e.to_string() })?;

        Ok(members.into_iter().map(|m| User {
            username: m.user.name.clone(),
            nickname: m.nick.unwrap_or(m.user.name),
            is_admin: false,
        }).collect())
    }

    async fn get_messages(
        &self,
        remote_channel_id: &str,
        limit: usize,
        _offset: usize,
    ) -> Result<Vec<Message>, DBridgeError> {
        let channel_id: u64 = remote_channel_id
            .parse()
            .map_err(|_| DBridgeError::Validation("Invalid Discord channel ID".to_string()))?;
        let channel_id = ChannelId::new(channel_id);

        let messages = channel_id.messages(&self.http, serenity::builder::GetMessages::new().limit(limit as u8)).await
            .map_err(|e| DBridgeError::Platform { platform: "discord".into(), message: e.to_string() })?;

        Ok(messages.into_iter().map(|msg| Message {
            id: msg.id.to_string(),
            sender: User {
                username: msg.author.name.clone(),
                nickname: msg.member.as_ref().and_then(|m| m.nick.clone()).unwrap_or(msg.author.name.clone()),
                is_admin: false,
            },
            content: msg.content.clone(),
            timestamp: msg.timestamp.unix_timestamp() as u64,
        }).collect())
    }

    async fn listen(&self, tx: mpsc::Sender<PlatformEvent>) -> Result<(), DBridgeError> {
        let intents = GatewayIntents::GUILD_MESSAGES 
            | GatewayIntents::DIRECT_MESSAGES 
            | GatewayIntents::MESSAGE_CONTENT
            | GatewayIntents::GUILD_MEMBERS;

        let event_producer = EventProducer::new(tx);
        let handler = DiscordHandler { event_producer };

        let mut client = Client::builder(&self.token, intents)
            .event_handler(handler)
            .await
            .map_err(|e| DBridgeError::Platform {
                platform: "discord".into(),
                message: e.to_string(),
            })?;

        client.start().await.map_err(|e| DBridgeError::Platform {
            platform: "discord".into(),
            message: e.to_string(),
        })
    }

    fn should_echo(&self) -> bool {
        false
    }
}

struct DiscordHandler {
    event_producer: EventProducer,
}

#[serenity_async_trait]
impl EventHandler for DiscordHandler {
    async fn message(&self, _ctx: Context, msg: SerenityMessage) {
        if msg.author.bot {
            return;
        }

        let user = User {
            username: msg.author.name.clone(),
            nickname: msg.member.as_ref().and_then(|m| m.nick.clone()).unwrap_or(msg.author.name.clone()),
            is_admin: false,
        };

        self.event_producer.emit_message(
            "discord".into(),
            msg.channel_id.to_string(),
            user,
            msg.content
        ).await;
    }

    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }
}
