use crate::config::database::DatabaseConfig;
use std::env;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub server_host: String,
    pub server_port: u16,
    pub jwt_secret: String,
    pub jwt_access_expiry: i64,
    pub jwt_refresh_expiry: i64,
    pub jwt_issuer: String,
    pub jwt_audience: String,
    pub confirm_code_expiry: i64,
    pub rust_log: String,
    pub db_config: DatabaseConfig,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        // Load .env file if it exists
        dotenvy::dotenv().ok();

        Ok(Self {
            database_url: env::var("DATABASE_URL")
                .map_err(|_| ConfigError::MissingEnvVar("DATABASE_URL".to_string()))?,
            server_host: env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidPort)?,
            jwt_secret: env::var("JWT_SECRET")
                .unwrap_or_else(|_| "dev-secret-change-in-production".to_string()),
            jwt_access_expiry: env::var("JWT_ACCESS_EXPIRY")
                .unwrap_or_else(|_| "3600".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidTokenExpiry)?,
            jwt_refresh_expiry: env::var("JWT_REFRESH_EXPIRY")
                .unwrap_or_else(|_| "604800".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidTokenExpiry)?,
            jwt_issuer: env::var("JWT_ISSUER").unwrap_or_else(|_| "axum-backend".to_string()),
            jwt_audience: env::var("JWT_AUDIENCE")
                .unwrap_or_else(|_| "axum-backend-api".to_string()),
            confirm_code_expiry: env::var("CONFIRMATION_CODE_EXPIRY")
                .unwrap_or_else(|_| "60".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidTokenExpiry)?,
            rust_log: env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
            db_config: DatabaseConfig::from_env(),
        })
    }

    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server_host, self.server_port)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Missing environment variable: {0}")]
    MissingEnvVar(String),

    #[error("Invalid port number")]
    InvalidPort,

    #[error("Invalid token expiry duration")]
    InvalidTokenExpiry,
}
