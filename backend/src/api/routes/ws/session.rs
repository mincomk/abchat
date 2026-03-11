use axum::extract::ws::{self, WebSocket};
use futures_util::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};
use std::time::Duration;
use tokio::time::interval;
use tracing::{info, instrument, warn};

use super::handler;
use crate::{
    AppResult, AppState, SessionState, WsError, WsPacketC2S, WsPacketS2C,
    service::pubsub::MessageSubscriber,
};

pub struct WsActor {
    state: AppState,
    channel_id: String,
    session_state: SessionState,
    sink: SplitSink<WebSocket, ws::Message>,
    stream: SplitStream<WebSocket>,
    subscriber: Option<Box<dyn MessageSubscriber>>,
}

impl WsActor {
    pub fn new(state: AppState, channel_id: String, socket: WebSocket) -> Self {
        let (sink, stream) = socket.split();
        Self {
            state,
            channel_id,
            session_state: SessionState::default(),
            sink,
            stream,
            subscriber: None,
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
                    if let Some(sub) = self.subscriber.as_ref() {
                        sub.next().await
                    } else {
                        std::future::pending().await
                    }
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
        info!("WebSocket session terminated.");
    }

    async fn handle_text_message(&mut self, text: String) -> AppResult<()> {
        let packet = match serde_json::from_str::<WsPacketC2S>(&text) {
            Ok(p) => p,
            Err(e) => {
                return self.send_error(&format!("Invalid packet: {e}")).await;
            }
        };

        match handler::handle_packet(
            &self.state,
            &mut self.session_state,
            packet,
            &self.channel_id,
        )
        .await
        {
            Ok(auth_success) => {
                if auth_success {
                    match self.state.pubsub.subscribe(&self.channel_id).await {
                        Ok(sub) => {
                            self.subscriber = Some(sub);
                        }
                        Err(e) => {
                            warn!("Failed to subscribe: {:?}", e);
                            self.send_error("Internal subscription error").await?;
                        }
                    }
                }
                Ok(())
            }
            Err(e) => {
                let msg = if let crate::AppError::Service(_) = e {
                    "Internal server error"
                } else {
                    &e.to_string()
                };
                self.send_error(msg).await
            }
        }
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
