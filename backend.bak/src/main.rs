mod config;

use dbridge_api::ApiState;
use dbridge_core::{Channel, Provider};
use dbridge_engine::BridgeEngine;
use dbridge_persistence::{InMemoryPersistence, Persistence};
use dotenvy::dotenv;
use provider_discord::DiscordProvider;
use provider_web::WebProvider;
use std::env;
use std::sync::Arc;
use tracing::{error, info};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

mod config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .init();

    info!("Starting DBridge Engine (Modular)...");

    // 1. Configuration
    let app_config = config::load().expect("Failed to load configuration (config.toml)");
    if app_config.web.jwt_secret == "your_secret_here" {
        error!("INSECURE JWT SECRET: Please change jwt_secret in config.toml!");
    }
    let discord_token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN must be set");

    // Initialize Persistence
    let persistence = Arc::new(InMemoryPersistence::new());

    // Bootstrap default admin account
    if let (Some(username), Some(password_hash)) = (
        &app_config.web.default_admin_user,
        &app_config.web.default_admin_password_hash,
    ) {
        info!("Bootstrapping default admin account: {}", username);
        let is_admin = app_config.web.admin_users.contains(username);
        let account = dbridge_persistence::Account {
            username: username.clone(),
            password_hash: password_hash.clone(),
            nickname: username.clone(),
            is_admin,
        };
        persistence.save_account(account).await?;
    }

    // 2. Initialize Engine
    let engine = Arc::new(BridgeEngine::new(persistence.clone()));

    // 3. Initialize API State
    let api_state = Arc::new(ApiState {
        persistence: persistence.clone(),
        jwt_secret: app_config.web.jwt_secret.clone(),
        admin_users: app_config.web.admin_users.clone(),
    });

    // 4. Initialize Providers
    let discord = Arc::new(DiscordProvider::new(discord_token));
    let web = Arc::new(WebProvider::new(
        app_config.web.address.clone(),
        app_config.web.jwt_secret.clone(),
        persistence,
        api_state,
    ));

    engine.add_platform(discord).await;
    engine.add_platform(web).await;

    // 5. Define the Bridge Channels from Config
    for chan_cfg in app_config.channels {
        info!("Configuring bridge: {}", chan_cfg.name);

        let master_remote_id = if chan_cfg.master.remote_id.starts_with('$') {
            env::var(&chan_cfg.master.remote_id[1..]).unwrap_or(chan_cfg.master.remote_id)
        } else if chan_cfg.master.remote_id == "DISCORD_CHANNEL_ID_HERE" {
            env::var("DISCORD_CHANNEL_ID").unwrap_or(chan_cfg.master.remote_id)
        } else {
            chan_cfg.master.remote_id
        };

        let mut channel = Channel::new(
            chan_cfg.id,
            chan_cfg.name,
            Provider {
                platform: chan_cfg.master.platform,
                remote_id: master_remote_id,
            },
        );

        for sec in chan_cfg.secondaries {
            channel.add_secondary_provider(Provider {
                platform: sec.platform,
                remote_id: sec.remote_id,
            });
        }

        engine.add_channel(channel).await;
    }

    // 6. Run
    if let Err(e) = engine.run().await {
        error!("Engine error: {}", e);
    }

    Ok(())
}
