use std::{process::exit, sync::Arc};

use backend::{
    AppConfig, AppState, api::router::create_router, chat::ChatManager,
    onboard::init_admin_account, persistence::postgres::PostgresPersistence,
    pubsub::redis::RedisMessagePubSub,
};
use tokio::net::TcpListener;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    tracing::info!("Hello. ABChat");

    ctrlc::set_handler(|| {
        tracing::info!("Bye.");
        exit(0);
    })?;

    let config = AppConfig::from_env()?;

    let pg = Arc::new(PostgresPersistence::connect(&config.postgres_url).await?);
    let redis = Arc::new(RedisMessagePubSub::connect(&config.redis_url).await?);

    pg.init_db().await?;

    let notification_service = Arc::new(backend::service::notifications::NotificationService::new(
        pg.clone(),
        config.vapid_public_key.clone(),
        config.vapid_private_key.clone(),
    ));

    tracing::info!("Infrastructure loaded");

    let state = AppState {
        persistence: pg.clone(),
        pubsub: redis.clone(),
        chat_manager: ChatManager::new(pg, redis, notification_service.clone()),
        notification_service,
        jwt_secret: config.jwt_secret.as_bytes().to_vec(),
        vapid_public_key: config.vapid_public_key.clone(),
        vapid_private_key: config.vapid_private_key.clone(),
    };


    init_admin_account(&config, &state).await?;

    let router = create_router(state);

    tracing::info!("Serving HTTP for {}", config.http_listen);

    let listener = TcpListener::bind(&config.http_listen).await?;
    axum::serve(listener, router).await?;

    Ok(())
}
