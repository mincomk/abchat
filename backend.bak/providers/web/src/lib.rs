use async_trait::async_trait;
use axum::{
    extract::{
        FromRef, Path, State,
        ws::{Message as WsMessage, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
    routing::get,
};
use dbridge_api::{ApiState, Claims};
use dbridge_core::{Message, Platform, PlatformEvent, User};
use dbridge_error::DBridgeError;
use dbridge_persistence::Persistence;
use dbridge_provider_sdk::EventProducer;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::{broadcast, mpsc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WebCommand {
    Identify { token: String },
    SendMessage { content: String },
}

pub struct WebChannelState {
    tx: broadcast::Sender<Message>,
}

pub struct WebProviderState {
    pub channels: Arc<Mutex<HashMap<String, WebChannelState>>>,
    pub persistence: Arc<dyn Persistence>,
    pub jwt_secret: String,
    pub api_state: Arc<ApiState>,
}

#[derive(Clone)]
pub struct WebProvider {
    state: Arc<WebProviderState>,
    addr: String,
}

impl WebProvider {
    pub fn new(
        addr: String,
        jwt_secret: String,
        persistence: Arc<dyn Persistence>,
        api_state: Arc<ApiState>,
    ) -> Self {
        Self {
            state: Arc::new(WebProviderState {
                channels: Arc::new(Mutex::new(HashMap::new())),
                persistence,
                jwt_secret,
                api_state,
            }),
            addr,
        }
    }

    fn get_or_create_channel(&self, channel_id: &str) -> broadcast::Sender<Message> {
        let mut channels = match self.state.channels.lock() {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("Mutex poisoned: {}", e);
                e.into_inner()
            }
        };
        if let Some(chan) = channels.get(channel_id) {
            chan.tx.clone()
        } else {
            let (tx, _) = broadcast::channel(100);
            channels.insert(channel_id.to_string(), WebChannelState { tx: tx.clone() });
            tx
        }
    }
}

#[async_trait]
impl Platform for WebProvider {
    fn name(&self) -> String {
        "web".to_string()
    }

    async fn send_message(
        &self,
        remote_channel_id: &str,
        message: &Message,
    ) -> Result<String, DBridgeError> {
        let tx = self.get_or_create_channel(remote_channel_id);
        if let Err(e) = tx.send(message.clone()) {
            tracing::debug!(
                "Failed to send broadcast message (likely no subscribers): {}",
                e
            );
        }
        Ok(message.id.clone())
    }

    async fn get_members(&self, _remote_channel_id: &str) -> Result<Vec<User>, DBridgeError> {
        Ok(vec![])
    }

    async fn get_messages(
        &self,
        remote_channel_id: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Message>, DBridgeError> {
        self.state
            .persistence
            .get_messages("web", remote_channel_id, limit, offset)
            .await
    }

    async fn listen(&self, tx: mpsc::Sender<PlatformEvent>) -> Result<(), DBridgeError> {
        let event_producer = EventProducer::new(tx);

        let web_state = Arc::new(WebState {
            provider_state: self.state.clone(),
            event_producer,
        });

        let app = dbridge_api::create_router::<Arc<WebState>>()
            .route("/ws/{channel_id}", get(ws_handler))
            .with_state(web_state);

        let listener = tokio::net::TcpListener::bind(&self.addr)
            .await
            .map_err(|e| DBridgeError::Internal(e.to_string()))?;

        tracing::info!(
            "Web provider listening on {} (Admin API + WebSocket)",
            self.addr
        );
        axum::serve(listener, app)
            .await
            .map_err(|e| DBridgeError::Internal(e.to_string()))
    }

    fn should_echo(&self) -> bool {
        false
    }
}

pub struct WebState {
    pub provider_state: Arc<WebProviderState>,
    pub event_producer: EventProducer,
}
/*
impl FromRef<Arc<WebState>> for Arc<ApiState> {
    fn from_ref(state: &Arc<WebState>) -> Self {
        state.provider_state.api_state.clone()
    }
}
*/

async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(channel_id): Path<String>,
    State(state): State<Arc<WebState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, channel_id, state))
}

async fn handle_socket(mut socket: WebSocket, channel_id: String, state: Arc<WebState>) {
    let session_id = Uuid::new_v4().to_string();
    let mut rx: Option<broadcast::Receiver<Message>> = None;
    let mut user: Option<User> = None;

    loop {
        tokio::select! {
            result = async {
                if let Some(r) = rx.as_mut() {
                    r.recv().await
                } else {
                    std::future::pending().await
                }
            } => {
                if let Ok(msg) = result {
                    if let Ok(json) = serde_json::to_string(&msg) {
                        if socket.send(WsMessage::Text(json.into())).await.is_err() {
                            break;
                        }
                    }
                }
            }
            msg = socket.recv() => {
                match msg {
                    Some(Ok(WsMessage::Text(text))) => {
                        if let Ok(cmd) = serde_json::from_str::<WebCommand>(&text) {
                            match cmd {
                                WebCommand::Identify { token } => {
                                    if user.is_some() {
                                        let _ = socket.send(WsMessage::Text("Already identified".into())).await;
                                        continue;
                                    }
                                    let decoding_key = DecodingKey::from_secret(state.provider_state.jwt_secret.as_bytes());
                                    let validation = Validation::new(Algorithm::HS256);

                                    match decode::<Claims>(&token, &decoding_key, &validation) {
                                        Ok(token_data) => {
                                            let username = token_data.claims.sub;

                                            let account = match state.provider_state.persistence.get_account(&username).await {
                                                Ok(Some(a)) => a,
                                                _ => {
                                                    let _ = socket.send(WsMessage::Text("Unauthorized username".into())).await;
                                                    break;
                                                }
                                            };

                                            let u = User {
                                                username,
                                                nickname: account.nickname.clone(),
                                                is_admin: account.is_admin,
                                            };
                                            user = Some(u.clone());

                                            state.event_producer.emit_member_joined(
                                                "web".into(),
                                                channel_id.clone(),
                                                session_id.clone(),
                                                u
                                            ).await;

                                            rx = Some({
                                                let mut channels = match state.provider_state.channels.lock() {
                                                    Ok(c) => c,
                                                    Err(e) => e.into_inner(),
                                                };
                                                channels.entry(channel_id.clone())
                                                    .or_insert_with(|| {
                                                        let (tx, _) = broadcast::channel(100);
                                                        WebChannelState { tx }
                                                    })
                                                    .tx.subscribe()
                                            });

                                            tracing::info!("User identified: {} (@{})", account.nickname, user.as_ref().unwrap().username);
                                        },
                                        Err(e) => {
                                            tracing::error!("Invalid JWT: {}", e);
                                            let _ = socket.send(WsMessage::Text("Invalid token".into())).await;
                                            break;
                                        }
                                    }
                                }
                                WebCommand::SendMessage { content } => {
                                    if let Some(ref u) = user {
                                        state.event_producer.emit_message(
                                            "web".into(),
                                            channel_id.clone(),
                                            u.clone(),
                                            content
                                        ).await;
                                    } else {
                                        let _ = socket.send(WsMessage::Text("Identify first".into())).await;
                                    }
                                }
                            }
                        }
                    }
                    _ => break,
                }
            }
        }
    }

    if let Some(u) = user {
        state
            .event_producer
            .emit_member_left("web".into(), channel_id.clone(), session_id.clone(), u)
            .await;
    }
}
