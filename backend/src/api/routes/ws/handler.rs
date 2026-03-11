use uuid::Uuid;

use crate::{
    AppError, AppResult, AppState, Message, MessageUser, User, UserError, WsPacketC2S,
    auth::AuthError, chat::ChatSession, util,
};

pub async fn handle_packet(
    state: &AppState,
    user: &User,
    chat: &ChatSession,
    packet: WsPacketC2S,
    channel_id: &str,
) -> AppResult<()> {
    match packet {
        WsPacketC2S::Identify(_) => Err(AppError::Auth(AuthError::AlreadyAuthenticated)),
        WsPacketC2S::SendMessage(msg) => {
            validate_message(&msg.content)?;

            let user = state
                .persistence
                .get_user(&user.username)
                .await?
                .ok_or(AuthError::Unauthorized)?;

            let message = Message {
                id: Uuid::new_v4().to_string(),
                channel_id: channel_id.to_string(),
                content: msg.content,
                timestamp: util::now_u64(),
                sender: MessageUser {
                    username: user.username,
                    nickname: user.nickname,
                },
            };

            chat.send(message).await;

            Ok(())
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
