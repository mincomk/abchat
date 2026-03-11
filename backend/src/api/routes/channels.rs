use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::Deserialize;

use crate::{AppResult, AppState, Message, User};

#[derive(Debug, Deserialize)]
pub struct ListMessagesQuery {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[utoipa::path(
    get,
    path = "/channels/{channel_id}/messages",
    tag = "channels",
    security(("bearer_auth" = [])),
    params(
        ("channel_id" = String, Path, description = "Channel ID"),
        ("limit" = Option<u32>, Query, description = "Max number of messages"),
        ("offset" = Option<u32>, Query, description = "Pagination offset")
    ),
    responses(
        (status = OK, description = "List of messages", body = Vec<Message>),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn list_messages(
    State(state): State<AppState>,
    _user: User,
    Path(channel_id): Path<String>,
    Query(query): Query<ListMessagesQuery>,
) -> AppResult<Json<Vec<Message>>> {
    let limit = query.limit.unwrap_or(50).min(100);
    let offset = query.offset.unwrap_or(0);

    let messages = state
        .persistence
        .list_messages(channel_id, limit, offset)
        .await?;

    Ok(Json(messages))
}
