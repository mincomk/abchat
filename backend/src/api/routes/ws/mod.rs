pub mod handler;
pub mod handshake;
pub mod session;

use crate::{AppResult, AppState, WsError, WsPacketS2C, api::handshake::handshake};
use axum::{
    extract::{
        Path, State, WebSocketUpgrade,
        ws::{self, WebSocket},
    },
    response::IntoResponse,
};
use session::WsActor;

pub async fn ws_route(
    State(state): State<AppState>,
    Path(channel_id): Path<String>,
    upgrade: WebSocketUpgrade,
) -> impl IntoResponse {
    upgrade.on_upgrade(
        async move |mut socket| match handshake(&state, &mut socket).await {
            Ok(user) => {
                let chat = state.chat_manager.subscribe(&channel_id, true).await;

                let actor = WsActor::new(state, chat, user, channel_id, socket);
                actor.run().await;
            }
            Err(e) => {
                tracing::warn!("Handshake failure: {:?}", e);
                if let Err(e1) = send_error(&mut socket, "Handshake failure").await {
                    tracing::warn!("Send failed while handshaking: {:?}", e1);
                }
            }
        },
    )
}

async fn send_packet(socket: &mut WebSocket, packet: WsPacketS2C) -> AppResult<()> {
    let text = serde_json::to_string(&packet)?;
    socket
        .send(ws::Message::Text(text.into()))
        .await
        .map_err(|e| crate::ServiceError::Internal(e.to_string()).into())
}

async fn send_error(socket: &mut WebSocket, message: &str) -> AppResult<()> {
    let packet = WsPacketS2C::Error(WsError {
        message: message.to_string(),
    });
    send_packet(socket, packet).await
}
