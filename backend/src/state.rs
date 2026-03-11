use crate::{chat::ChatManager, persistence::Persistence, pubsub::MessagePubSub};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub persistence: Arc<dyn Persistence>,
    pub pubsub: Arc<dyn MessagePubSub>,
    pub chat_manager: ChatManager,
    pub jwt_secret: Vec<u8>,
}
