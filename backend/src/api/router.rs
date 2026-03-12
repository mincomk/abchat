use axum::{routing::get, Router};

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
        .routes(routes!(list_users))
        .routes(routes!(delete_user))
        .routes(routes!(login_handler))
        .routes(routes!(change_password_handler))
        .routes(routes!(change_nickname_handler))
        .routes(routes!(admin_change_password))
        .routes(routes!(admin_change_nickname))
        .routes(routes!(update_user_admin))
        .routes(routes!(list_messages))
        .routes(routes!(register_user))
        .routes(routes!(subscribe_handler))
        .routes(routes!(unsubscribe_handler))
        .routes(routes!(get_settings_handler))
        .routes(routes!(update_settings_handler))
        .routes(routes!(get_vapid_key_handler))
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
