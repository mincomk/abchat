pub mod handler;
pub mod session;

use axum::{
    extract::{Path, State, WebSocketUpgrade},
    response::IntoResponse,
};
use crate::AppState;
use session::WsSession;

pub async fn ws_route(
    State(state): State<AppState>,
    Path(channel_id): Path<String>,
    upgrade: WebSocketUpgrade,
) -> impl IntoResponse {
    upgrade.on_upgrade(async move |socket| {
        let session = WsSession::new(state, channel_id);
        session.handle(socket).await;
    })
}
