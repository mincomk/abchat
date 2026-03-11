use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub jwt_secret: String,
    pub postgres_url: String,
    pub redis_url: String,
    pub http_listen: String,

    pub admin_username: Option<String>,
    pub admin_password_hash: Option<String>,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let _ = dotenvy::dotenv();
        let config: AppConfig = envy::from_env()?;

        Ok(config)
    }
}
