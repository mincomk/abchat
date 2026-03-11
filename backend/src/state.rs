use crate::{chat::ChatManager, persistence::Persistence, service::pubsub::MessagePubSub};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub persistence: Arc<dyn Persistence>,
    pub chat_manager: ChatManager,
    pub jwt_secret: Vec<u8>,
}
