use axum::{Router, routing::get};

use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use utoipa::openapi::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_swagger_ui::SwaggerUi;

use crate::AppState;

use super::routes::*;

pub fn create_router(state: AppState) -> Router {
    let (router, api): (Router<_>, OpenApi) = OpenApiRouter::new()
        .routes(routes!(list_users, delete_user, login_handler,))
        .routes(routes!(list_messages))
        .routes(routes!(register_user))
        .split_for_parts();

    let swagger = SwaggerUi::new("/swagger-ui").url("/openapi.json", api.clone());

    // TODO fix cors
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let router = router
        .route("/ws/{channel_id}", get(ws_route))
        .merge(swagger)
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    let router = Router::new().nest("/api", router);

    router.with_state(state)
}
