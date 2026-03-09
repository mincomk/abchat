use dbridge_core::{PlatformEvent, Message, User};
use dbridge_error::DBridgeError;
use tokio::sync::mpsc;
use tracing::error;
use uuid::Uuid;

#[derive(Clone)]
pub struct EventProducer {
    tx: mpsc::Sender<PlatformEvent>,
}

impl EventProducer {
    pub fn new(tx: mpsc::Sender<PlatformEvent>) -> Self {
        Self { tx }
    }

    pub async fn emit_message(&self, platform: String, remote_channel_id: String, user: User, content: String) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let event = PlatformEvent::MessageReceived {
            platform,
            remote_channel_id,
            message: Message {
                id: Uuid::new_v4().to_string(),
                sender: user,
                content,
                timestamp,
            },
        };

        if let Err(e) = self.tx.send(event).await {
            error!("Failed to emit MessageReceived event: {}", e);
        }
    }

    pub async fn emit_member_joined(&self, platform: String, remote_channel_id: String, identifier: String, user: User) {
        let event = PlatformEvent::MemberJoined {
            platform,
            remote_channel_id,
            identifier,
            user,
        };
        if let Err(e) = self.tx.send(event).await {
            error!("Failed to emit MemberJoined event: {}", e);
        }
    }

    pub async fn emit_member_left(&self, platform: String, remote_channel_id: String, identifier: String, user: User) {
        let event = PlatformEvent::MemberLeft {
            platform,
            remote_channel_id,
            identifier,
            user,
        };
        if let Err(e) = self.tx.send(event).await {
            error!("Failed to emit MemberLeft event: {}", e);
        }
    }
}

pub mod utils {
    /// Sanitizes content for platforms (basic example)
    pub fn sanitize_content(content: &str) -> String {
        content.trim().to_string()
    }
}
