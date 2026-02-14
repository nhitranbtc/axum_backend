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

    // gRPC Configuration
    pub grpc_port: u16,
    pub grpc_reflection_enabled: bool,
    pub grpc_web_enabled: bool,
    pub grpc_max_connections: usize,
    pub grpc_actor_pool_size: usize,
    pub grpc_cors_origins: Vec<String>,

    // Redis Configuration
    pub redis_url: String,
    pub redis_pool_size: usize,

    // NATS Configuration
    pub nats_url: String,
    pub nats_user: Option<String>,
    pub nats_password: Option<String>,
    pub nats_token: Option<String>,
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

            // gRPC Configuration
            grpc_port: env::var("GRPC_PORT")
                .unwrap_or_else(|_| "50051".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidPort)?,
            grpc_reflection_enabled: env::var("GRPC_REFLECTION_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            grpc_web_enabled: env::var("GRPC_WEB_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            grpc_max_connections: env::var("GRPC_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()
                .unwrap_or(1000),
            grpc_actor_pool_size: env::var("GRPC_ACTOR_POOL_SIZE")
                .unwrap_or_else(|_| "20".to_string())
                .parse()
                .unwrap_or(20),
            grpc_cors_origins: env::var("GRPC_CORS_ORIGINS")
                .unwrap_or_else(|_| "*".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),

            // Redis Configuration
            redis_url: env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://127.0.0.1:6379/".to_string()),
            redis_pool_size: env::var("REDIS_POOL_SIZE")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),

            // NATS Configuration
            nats_url: env::var("NATS_URL").unwrap_or_else(|_| "nats://127.0.0.1:4222".to_string()),
            nats_user: env::var("NATS_USER").ok(),
            nats_password: env::var("NATS_PASSWORD").ok(),
            nats_token: env::var("NATS_TOKEN").ok(),
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
