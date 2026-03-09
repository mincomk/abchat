use crate::{persistence::Persistence, service::pubsub::MessagePubSub};
use std::sync::Arc;

pub mod api;
pub mod auth;

pub mod types;
pub use types::*;

pub mod error;
pub use error::*;

pub mod config;
pub use config::*;

pub mod service;
pub use service::*;

pub mod util;

#[derive(Clone)]
pub struct AppState {
    pub persistence: Arc<dyn Persistence>,
    pub pubsub: Arc<dyn MessagePubSub>,
    pub config: AppConfig,
}
