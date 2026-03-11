use std::sync::Arc;

use backend::{
    AppConfig, AppState, api::router::create_router, persistence::postgres::PostgresPersistence,
    pubsub::redis::RedisMessagePubSub,
};
use git_version::git_version;
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

const GIT_VERSION: &str = git_version!();

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    tracing::info!("Hello. ABChat {}", GIT_VERSION);

    let config = AppConfig::from_env()?;

    let pg = PostgresPersistence::connect(&config.postgres_url).await?;
    let redis = RedisMessagePubSub::connect(&config.redis_url).await?;

    tracing::info!("Infrastructure loaded");

    let state = AppState {
        persistence: Arc::new(pg),
        pubsub: Arc::new(redis),
        jwt_secret: config.jwt_secret.as_bytes().to_vec(),
    };

    let router = create_router(state);

    tracing::info!("Serving HTTP for {}", config.http_listen);

    let listener = TcpListener::bind(&config.http_listen).await?;
    axum::serve(listener, router).await?;

    Ok(())
}
