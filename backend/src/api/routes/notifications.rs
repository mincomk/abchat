use axum::{
    Json,
    extract::State,
    http::StatusCode,
};

use crate::{
    AppResult, AppState, Subscription, SubscriptionRequest, User, UserSettingsRequest,
    UserSettingsResponse, VapidPublicKeyResponse,
};

#[utoipa::path(
    post,
    path = "/notifications/subscribe",
    tag = "notifications",
    security(("bearer_auth" = [])),
    request_body = SubscriptionRequest,
    responses(
        (status = 200, description = "Subscription saved"),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn subscribe_handler(
    State(state): State<AppState>,
    user: User,
    Json(payload): Json<SubscriptionRequest>,
) -> AppResult<StatusCode> {
    let sub = Subscription {
        username: user.username,
        endpoint: payload.endpoint,
        p256dh: payload.p256dh,
        auth: payload.auth,
    };

    state.persistence.add_subscription(sub).await?;

    Ok(StatusCode::OK)
}

#[utoipa::path(
    post,
    path = "/notifications/unsubscribe",
    tag = "notifications",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "All subscriptions removed"),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn unsubscribe_handler(
    State(state): State<AppState>,
    user: User,
) -> AppResult<StatusCode> {
    state
        .persistence
        .delete_user_subscriptions(&user.username)
        .await?;

    Ok(StatusCode::OK)
}

#[utoipa::path(
    get,
    path = "/notifications/settings",
    tag = "notifications",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "User settings", body = UserSettingsResponse),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn get_settings_handler(
    State(state): State<AppState>,
    user: User,
) -> AppResult<Json<UserSettingsResponse>> {
    let mode = state
        .persistence
        .get_user_notification_mode(&user.username)
        .await?;

    Ok(Json(UserSettingsResponse {
        notification_mode: mode,
    }))
}

#[utoipa::path(
    put,
    path = "/notifications/settings",
    tag = "notifications",
    security(("bearer_auth" = [])),
    request_body = UserSettingsRequest,
    responses(
        (status = 200, description = "Settings updated"),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn update_settings_handler(
    State(state): State<AppState>,
    user: User,
    Json(payload): Json<UserSettingsRequest>,
) -> AppResult<StatusCode> {
    state
        .persistence
        .set_user_notification_mode(&user.username, payload.notification_mode)
        .await?;

    Ok(StatusCode::OK)
}

#[utoipa::path(
    get,
    path = "/notifications/vapid-key",
    tag = "notifications",
    responses(
        (status = 200, description = "VAPID public key", body = VapidPublicKeyResponse),
    )
)]
pub async fn get_vapid_key_handler(State(state): State<AppState>) -> Json<VapidPublicKeyResponse> {
    Json(VapidPublicKeyResponse {
        public_key: state.vapid_public_key,
    })
}
