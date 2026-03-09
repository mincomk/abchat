use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub cache: CacheConfig,
    pub web: WebConfig,
    pub channels: Vec<ChannelConfig>,
}

#[derive(Debug, Deserialize)]
pub struct CacheConfig {
    pub driver: String, // "in-memory" or "valkey"
    pub url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct WebConfig {
    pub address: String,
    pub jwt_secret: String,
    pub admin_users: Vec<String>,
    pub default_admin_user: Option<String>,
    pub default_admin_password_hash: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChannelConfig {
    pub id: String,
    pub name: String,
    pub master: ProviderConfig,
    pub secondaries: Vec<ProviderConfig>,
}

#[derive(Debug, Deserialize)]
pub struct ProviderConfig {
    pub platform: String,
    pub remote_id: String,
}

pub fn load() -> Result<AppConfig, config::ConfigError> {
    config::Config::builder()
        .add_source(config::File::with_name("config").required(false))
        .add_source(config::File::with_name("config.toml").required(false))
        .add_source(config::Environment::with_prefix("DBRIDGE"))
        .build()?
        .try_deserialize()
}
