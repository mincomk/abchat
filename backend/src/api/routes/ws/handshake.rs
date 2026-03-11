use axum::extract::ws::{self, WebSocket};

use crate::{
    AppError, AppResult, AppState, User, WsPacketC2S,
    auth::{AuthError, jwt},
};

pub async fn handshake(state: &AppState, ws: &mut WebSocket) -> AppResult<User> {
    if let Some(Ok(ws::Message::Text(text))) = ws.recv().await {
        let packet: WsPacketC2S = serde_json::from_str(&text)
            .map_err(Into::into)
            .map_err(AppError::Service)?;

        if let WsPacketC2S::Identify(creds) = packet {
            jwt::auth_user(&state, &creds.token).await
        } else {
            Err(AppError::Auth(AuthError::Unauthorized))
        }
    } else {
        Err(AppError::Auth(AuthError::Unauthorized))
    }
}
