use axum::extract::ws::{self, WebSocket};
use futures_util::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};
use std::time::Duration;
use tokio::time::interval;
use tracing::{instrument, warn};

use super::handler;
use crate::{AppResult, AppState, User, WsError, WsPacketC2S, WsPacketS2C, chat::ChatSession};

pub struct WsActor {
    state: AppState,
    channel_id: String,
    chat: ChatSession,
    user: User,
    sink: SplitSink<WebSocket, ws::Message>,
    stream: SplitStream<WebSocket>,
}

impl WsActor {
    pub fn new(
        state: AppState,
        chat: ChatSession,
        user: User,
        channel_id: String,
        socket: WebSocket,
    ) -> Self {
        let (sink, stream) = socket.split();
        Self {
            state,
            channel_id,
            chat,
            user,
            sink,
            stream,
        }
    }

    #[instrument(skip_all, fields(channel = %self.channel_id))]
    pub async fn run(mut self) {
        let mut heartbeat = interval(Duration::from_secs(30));

        loop {
            tokio::select! {
                _ = heartbeat.tick() => {
                    if let Err(e) = self.sink.send(ws::Message::Ping(vec![].into())).await {
                        warn!("Failed to send ping: {:?}", e);
                        break;
                    }
                }

                res = async {
                    self.chat.recv().await
                } => {
                    match res {
                        Ok(msg) => {
                            let packet = WsPacketS2C::Message(msg);
                            if let Err(e) = self.send_packet(packet).await {
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

                msg = self.stream.next() => {
                    match msg {
                        Some(Ok(ws::Message::Text(text))) => {
                            if let Err(e) = self.handle_text_message(text.to_string()).await {
                                warn!("Error handling message: {:?}", e);
                            }
                        }
                        Some(Ok(ws::Message::Close(_))) => break,
                        Some(Ok(ws::Message::Ping(p))) => {
                            let _ = self.sink.send(ws::Message::Pong(p)).await;
                        }
                        Some(Ok(_)) => (),
                        Some(Err(e)) => {
                            tracing::debug!("WebSocket error: {:?}", e);
                            break;
                        }
                        None => break,
                    }
                }
            }
        }
        tracing::debug!("WebSocket session terminated.");
    }

    async fn handle_text_message(&mut self, text: String) -> AppResult<()> {
        let packet = match serde_json::from_str::<WsPacketC2S>(&text) {
            Ok(p) => p,
            Err(e) => {
                return self.send_error(&format!("Invalid packet: {e}")).await;
            }
        };

        if let Err(e) = handler::handle_packet(
            &self.state,
            &self.user,
            &self.chat,
            packet,
            &self.channel_id,
        )
        .await
        {
            let msg = if let crate::AppError::Service(_) = e {
                "Internal server error"
            } else {
                &e.to_string()
            };
            let _ = self.send_error(msg).await;
        }

        Ok(())
    }

    async fn send_packet(&mut self, packet: WsPacketS2C) -> AppResult<()> {
        let text = serde_json::to_string(&packet)?;
        self.sink
            .send(ws::Message::Text(text.into()))
            .await
            .map_err(|e| crate::ServiceError::Internal(e.to_string()).into())
    }

    async fn send_error(&mut self, message: &str) -> AppResult<()> {
        let packet = WsPacketS2C::Error(WsError {
            message: message.to_string(),
        });
        self.send_packet(packet).await
    }
}
