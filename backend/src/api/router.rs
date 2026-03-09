use axum::{Router, routing::get};
use utoipa::openapi::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::AppState;

use super::routes::*;

pub fn create_router(state: AppState) -> Router<AppState> {
    let (router, api): (Router<_>, OpenApi) = OpenApiRouter::new()
        .with_state(state)
        .routes(routes!(list_users, delete_user))
        .split_for_parts();

    let router = router
        .route(
            "openapi.json",
            get(api.to_json().expect("Failed to create OpenAPI")),
        )
        .route("/ws/{channel_id}", get(ws_route));

    router
}
