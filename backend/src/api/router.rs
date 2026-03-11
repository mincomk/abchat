use axum::{routing::get, Router};

use tower_http::trace::TraceLayer;
use utoipa::openapi::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_swagger_ui::SwaggerUi;

use crate::AppState;

use super::routes::*;

pub fn create_router(state: AppState) -> Router {
    let (router, api): (Router<_>, OpenApi) = OpenApiRouter::new()
        .routes(routes!(
            list_users,
            delete_user,
            login_handler,
            register_user,
            list_messages
        ))
        .split_for_parts();

    let swagger = SwaggerUi::new("/swagger-ui").url("/openapi.json", api.clone());

    let router = router
        .route(
            "openapi.json",
            get(api.to_json().expect("Failed to create OpenAPI")),
        )
        .route("/ws/{channel_id}", get(ws_route))
        .merge(swagger)
        .layer(TraceLayer::new_for_http());

    router.with_state(state)
}
