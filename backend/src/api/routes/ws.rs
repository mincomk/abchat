use std::sync::Arc;

use axum::{
    extract::{
        Path, State, WebSocketUpgrade,
        ws::{self, WebSocket},
    },
    response::IntoResponse,
};
use futures_util::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};
use serde::de::DeserializeOwned;
use tokio::sync::Mutex;
use tracing::instrument;
use uuid::Uuid;

use crate::{
    AppResult, AppState, Message, SessionState, UserError, WsError, WsPacketC2S, WsPacketS2C,
    auth::{AuthError, jwt},
    util,
};

type ArcSender = Arc<Mutex<SplitSink<WebSocket, ws::Message>>>;
type ArcSessionState = Arc<Mutex<SessionState>>;

pub async fn ws_route(
    State(state): State<AppState>,
    Path(channel_id): Path<String>,
    upgrade: WebSocketUpgrade,
) -> impl IntoResponse {
    upgrade.on_upgrade(async move |socket| handle_socket(socket, state, channel_id).await)
}

#[instrument(level = "info", name = "Handling {session_state}", skip_all)]
async fn handle_socket(socket: WebSocket, state: AppState, channel_id: String) {
    let session_state = Arc::new(Mutex::new(SessionState::default()));

    let (sender, receiver) = socket.split();
    let sender = Arc::new(Mutex::new(sender));

    run_receiver_loop(receiver, sender, state, session_state, channel_id).await;
}

async fn spawn_sender(sender: ArcSender, state: AppState, channel_id: String) {
    tokio::spawn(async move {
        run_sender_loop(sender, state, channel_id).await;
    });
}

#[instrument(level = "info", name = "Handling {session_state}", skip_all)]
async fn run_sender_loop(sender: ArcSender, state: AppState, channel_id: String) {
    match state.pubsub.subscribe(&channel_id).await {
        Ok(subscriber) => loop {
            if let Ok(msg) = subscriber.next().await {
                if let Err(_) = send(sender.clone(), WsPacketS2C::Message(msg)).await {
                    break;
                }
            } else {
                tracing::warn!("Subscriber closed.");
                break;
            }
        },
        Err(e) => {
            tracing::warn!("Failed to subscribe: {:?}", e);
        }
    }
}

#[instrument(level = "info", name = "Handling {session_state}", skip_all)]
async fn run_receiver_loop(
    mut receiver: SplitStream<WebSocket>,
    sender: ArcSender,
    state: AppState,
    session_state: ArcSessionState,
    channel_id: String,
) {
    loop {
        match receiver.next().await {
            Some(Ok(msg)) => match decode_message::<WsPacketC2S>(msg) {
                Ok(data) => {
                    if let Err(e) = handle_message(
                        session_state.clone(),
                        state.clone(),
                        sender.clone(),
                        data,
                        &channel_id,
                    )
                    .await
                    {
                        tracing::warn!("Server error while receiving in websocket: {:?}", e);
                        if let Err(_) = send_error(sender.clone(), "Internal Server Error").await {
                            break;
                        }
                    }
                }
                Err(msg) => {
                    if let Err(_) = send_error(sender.clone(), &msg).await {
                        break;
                    }
                }
            },
            Some(Err(err)) => {
                tracing::warn!("Error in socket {:?}", err,);
                break;
            }
            None => {
                tracing::info!("Closing socket");
                break;
            }
        }
    }
}

async fn handle_message(
    ss_arc: Arc<Mutex<SessionState>>,
    state: AppState,
    sender: ArcSender,
    data: WsPacketC2S,
    channel_id: &str,
) -> AppResult<()> {
    let session_state = ss_arc.lock().await.clone();
    match data {
        WsPacketC2S::Identify(identify) => {
            if session_state.username.is_none() {
                let user = jwt::auth_user(&state, &identify.token).await?;
                ss_arc.lock().await.username.replace(user.username);

                spawn_sender(sender.clone(), state.clone(), channel_id.to_string()).await;
            } else {
                return Err(AuthError::AlreadyAuthenticated.into());
            }
        }
        WsPacketC2S::SendMessage(message) => {
            if session_state.username.is_none() {
                return Err(AuthError::Unauthorized.into());
            }

            validate_message(&message.content)?;

            let message = Message {
                id: Uuid::new_v4().to_string(),
                channel_id: channel_id.to_string(),
                content: message.content,
                timestamp: util::now_u64(),
            };

            state.persistence.add_message(message.clone()).await?;

            if let Err(e) = state.pubsub.publish(&channel_id, message).await {
                tracing::warn!("Failed to publish message: {e}");
            }
        }
    }

    Ok(())
}

fn validate_message(content: &str) -> Result<(), UserError> {
    // Discord max message len
    if content.len() < 2000 {
        Ok(())
    } else {
        Err(UserError::MessageValidationFailed(
            "Message is too long. (max 2000)".to_string(),
        ))
    }
}

async fn send(sender: ArcSender, value: WsPacketS2C) -> Result<(), String> {
    let text = match serde_json::to_string(&value) {
        Ok(v) => v,
        Err(_) => {
            tracing::warn!("Serialization failure: {:?}", value);
            return Err("Serialization failure".into());
        }
    };

    sender
        .lock()
        .await
        .send(ws::Message::Text(text.into()))
        .await
        .map_err(|e| {
            tracing::warn!("WebSocket send failed: {:?}", e);
            "WebSocket send failed"
        })?;

    Ok(())
}

async fn send_error(sender: ArcSender, message: &str) -> Result<(), String> {
    send(
        sender,
        WsPacketS2C::Error(WsError {
            message: message.to_string(),
        }),
    )
    .await
}

fn decode_message<T>(msg: ws::Message) -> Result<T, String>
where
    T: DeserializeOwned,
{
    let text = msg.to_text().map_err(|_| "Malformed text".to_string())?;
    let value = serde_json::from_str::<T>(text).map_err(|_| "Malformed JSON".to_string())?;

    Ok(value)
}
