use crate::{
    chat::ChatManager, persistence::Persistence, pubsub::MessagePubSub,
    service::notifications::NotificationService,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub persistence: Arc<dyn Persistence>,
    pub pubsub: Arc<dyn MessagePubSub>,
    pub chat_manager: ChatManager,
    pub notification_service: Arc<NotificationService>,
    pub jwt_secret: Vec<u8>,
    pub vapid_public_key: String,
    pub vapid_private_key: String,
}
