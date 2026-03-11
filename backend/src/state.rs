use crate::{persistence::Persistence, service::pubsub::MessagePubSub};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub persistence: Arc<dyn Persistence>,
    pub pubsub: Arc<dyn MessagePubSub>,
    pub jwt_secret: Vec<u8>,
}
