use uuid::Uuid;

use crate::{
    AppResult, AppState, Message, SessionState, UserError, WsPacketC2S,
    auth::{AuthError, jwt},
    util,
};

pub async fn handle_packet(
    state: &AppState,
    session: &mut SessionState,
    packet: WsPacketC2S,
    channel_id: &str,
) -> AppResult<bool> {
    match packet {
        WsPacketC2S::Identify(identify) => {
            if session.username.is_none() {
                let user = jwt::auth_user(state, &identify.token).await?;
                session.username.replace(user.username);
                Ok(true)
            } else {
                Err(AuthError::AlreadyAuthenticated.into())
            }
        }
        WsPacketC2S::SendMessage(msg) => {
            let username = session.username.as_ref().ok_or(AuthError::Unauthorized)?;

            validate_message(&msg.content)?;

            let user = state
                .persistence
                .get_user(username)
                .await?
                .ok_or(AuthError::Unauthorized)?;

            let message = Message {
                id: Uuid::new_v4().to_string(),
                channel_id: channel_id.to_string(),
                content: msg.content,
                timestamp: util::now_u64(),
                sender: user,
            };

            state.persistence.add_message(message.clone()).await?;

            if let Err(e) = state.pubsub.publish(channel_id, message).await {
                tracing::warn!("Failed to publish message: {e}");
            }

            Ok(false)
        }
    }
}

fn validate_message(content: &str) -> Result<(), UserError> {
    if content.is_empty() {
        return Err(UserError::MessageValidationFailed(
            "Message cannot be empty".to_string(),
        ));
    }
    
    if content.len() < 2000 {
        Ok(())
    } else {
        Err(UserError::MessageValidationFailed(
            "Message is too long. (max 2000)".to_string(),
        ))
    }
}
