use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::interval;
use futures_util::{SinkExt, StreamExt, stream::{SplitSink, SplitStream}};
use axum::extract::ws::{self, WebSocket};
use tracing::{info, warn, instrument};

use crate::{
    AppState, SessionState, WsError, WsPacketC2S, WsPacketS2C,
    AppResult,
};
use super::handler;

pub struct WsSession {
    state: AppState,
    channel_id: String,
    session_state: Arc<Mutex<SessionState>>,
}

impl WsSession {
    pub fn new(state: AppState, channel_id: String) -> Self {
        Self {
            state,
            channel_id,
            session_state: Arc::new(Mutex::new(SessionState::default())),
        }
    }

    #[instrument(skip(self, socket), fields(channel = %self.channel_id))]
    pub async fn handle(self, socket: WebSocket) {
        let (sender, receiver) = socket.split();
        let sender = Arc::new(Mutex::new(sender));

        let s_state = self.state.clone();
        let c_id = self.channel_id.clone();
        let ss = self.session_state.clone();
        let snd = sender.clone();

        tokio::spawn(async move {
            Self::run_receiver_loop(receiver, snd, s_state, ss, c_id).await;
        });

        // We don't spawn the sender loop yet because we need to wait for authentication.
        // The receiver loop will call `spawn_sender` upon successful identification.
    }

    async fn run_receiver_loop(
        mut receiver: SplitStream<WebSocket>,
        sender: Arc<Mutex<SplitSink<WebSocket, ws::Message>>>,
        state: AppState,
        session_state: Arc<Mutex<SessionState>>,
        channel_id: String,
    ) {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(ws::Message::Text(text)) => {
                    let packet = match serde_json::from_str::<WsPacketC2S>(&text) {
                        Ok(p) => p,
                        Err(e) => {
                            let _ = Self::send_error(sender.clone(), &format!("Invalid packet: {e}")).await;
                            continue;
                        }
                    };

                    let sender_clone = sender.clone();
                    let state_clone = state.clone();
                    let channel_clone = channel_id.clone();

                    let res = handler::handle_packet(
                        &state,
                        session_state.clone(),
                        packet,
                        &channel_id,
                        move || {
                            // On Auth Success, spawn the sender loop and heartbeat
                            let snd = sender_clone.clone();
                            let st = state_clone.clone();
                            let ch = channel_clone.clone();
                            tokio::spawn(async move {
                                Self::run_sender_loop(snd, st, ch).await;
                            });
                        }
                    ).await;

                    if let Err(e) = res {
                        warn!("Error handling packet: {:?}", e);
                        // Redact internal errors
                        let msg = if let crate::AppError::Service(_) = e {
                            "Internal server error"
                        } else {
                            &e.to_string()
                        };
                        let _ = Self::send_error(sender.clone(), msg).await;
                    }
                }
                Ok(ws::Message::Close(_)) => break,
                Ok(ws::Message::Ping(p)) => {
                    let _ = sender.lock().await.send(ws::Message::Pong(p)).await;
                }
                Ok(_) => (), // Ignore binary/pong
                Err(e) => {
                    warn!("WebSocket receiver error: {:?}", e);
                    break;
                }
            }
        }
        info!("WebSocket receiver loop terminated.");
    }

    async fn run_sender_loop(
        sender: Arc<Mutex<SplitSink<WebSocket, ws::Message>>>,
        state: AppState,
        channel_id: String,
    ) {
        let subscriber = match state.pubsub.subscribe(&channel_id).await {
            Ok(s) => s,
            Err(e) => {
                warn!("Failed to subscribe to channel {}: {:?}", channel_id, e);
                let _ = Self::send_error(sender, "Internal subscription error").await;
                return;
            }
        };

        let mut heartbeat = interval(Duration::from_secs(30));
        
        loop {
            tokio::select! {
                _ = heartbeat.tick() => {
                    let mut s = sender.lock().await;
                    if let Err(e) = s.send(ws::Message::Ping(vec![].into())).await {
                        warn!("Failed to send ping: {:?}", e);
                        break;
                    }
                }
                msg = subscriber.next() => {
                    match msg {
                        Ok(msg) => {
                            let packet = WsPacketS2C::Message(msg);
                            if let Err(e) = Self::send_packet(sender.clone(), packet).await {
                                warn!("Failed to send message packet: {:?}", e);
                                break;
                            }
                        }
                        Err(e) => {
                            warn!("PubSub subscriber error: {:?}", e);
                            break;
                        }
                    }
                }
            }
        }
        info!("WebSocket sender loop terminated.");
    }

    async fn send_packet(
        sender: Arc<Mutex<SplitSink<WebSocket, ws::Message>>>,
        packet: WsPacketS2C,
    ) -> AppResult<()> {
        let text = serde_json::to_string(&packet)?;
        sender.lock().await.send(ws::Message::Text(text.into())).await
            .map_err(|e| crate::ServiceError::Internal(e.to_string()).into())
    }

    async fn send_error(
        sender: Arc<Mutex<SplitSink<WebSocket, ws::Message>>>,
        message: &str,
    ) -> AppResult<()> {
        let packet = WsPacketS2C::Error(WsError {
            message: message.to_string(),
        });
        Self::send_packet(sender, packet).await
    }
}
