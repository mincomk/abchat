#[derive(Debug, Clone)]
pub struct AppConfig {
    pub jwt_secret: String,
    pub database_url: String,
    pub redis_url: String,
}
