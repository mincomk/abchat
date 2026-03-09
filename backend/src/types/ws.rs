use serde::{Deserialize, Serialize};

use crate::Message;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsSendMessage {
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsIdentify {
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsError {
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsPacketC2S {
    SendMessage(WsSendMessage),
    Identify(WsIdentify),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsPacketS2C {
    Message(Message),
    Error(WsError),
}
