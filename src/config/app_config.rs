use crate::config::scylla::ScyllaConfig;
use std::env;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub server_host: String,
    pub server_port: u16,
    pub jwt_secret: String,
    pub jwt_access_expiry: i64,
    pub jwt_refresh_expiry: i64,
    pub jwt_issuer: String,
    pub jwt_audience: String,
    pub confirm_code_expiry: i64,
    pub rust_log: String,
    pub scylla: ScyllaConfig,

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

        let scylla =
            ScyllaConfig::from_env().map_err(|e| ConfigError::ScyllaConfig(e.to_string()))?;

        Ok(Self {
            server_host: env::var("SERVER_HOST")
                .map_err(|_| ConfigError::MissingEnvVar("SERVER_HOST".to_string()))?,
            server_port: env::var("SERVER_PORT")
                .map_err(|_| ConfigError::MissingEnvVar("SERVER_PORT".to_string()))?
                .parse()
                .map_err(|_| ConfigError::InvalidPort)?,
            jwt_secret: env::var("JWT_SECRET")
                .map_err(|_| ConfigError::MissingEnvVar("JWT_SECRET".to_string()))?,
            jwt_access_expiry: env::var("JWT_ACCESS_EXPIRY")
                .map_err(|_| ConfigError::MissingEnvVar("JWT_ACCESS_EXPIRY".to_string()))?
                .parse()
                .map_err(|_| ConfigError::InvalidTokenExpiry)?,
            jwt_refresh_expiry: env::var("JWT_REFRESH_EXPIRY")
                .map_err(|_| ConfigError::MissingEnvVar("JWT_REFRESH_EXPIRY".to_string()))?
                .parse()
                .map_err(|_| ConfigError::InvalidTokenExpiry)?,
            jwt_issuer: env::var("JWT_ISSUER")
                .map_err(|_| ConfigError::MissingEnvVar("JWT_ISSUER".to_string()))?,
            jwt_audience: env::var("JWT_AUDIENCE")
                .map_err(|_| ConfigError::MissingEnvVar("JWT_AUDIENCE".to_string()))?,
            confirm_code_expiry: env::var("CONFIRMATION_CODE_EXPIRY")
                .map_err(|_| ConfigError::MissingEnvVar("CONFIRMATION_CODE_EXPIRY".to_string()))?
                .parse()
                .map_err(|_| ConfigError::InvalidTokenExpiry)?,
            rust_log: env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
            scylla,

            // gRPC Configuration
            grpc_port: env::var("GRPC_PORT")
                .map_err(|_| ConfigError::MissingEnvVar("GRPC_PORT".to_string()))?
                .parse()
                .map_err(|_| ConfigError::InvalidPort)?,
            grpc_reflection_enabled: env::var("GRPC_REFLECTION_ENABLED")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            grpc_web_enabled: env::var("GRPC_WEB_ENABLED")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            grpc_max_connections: env::var("GRPC_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "100".to_string())
                .parse()
                .unwrap_or(100),
            grpc_actor_pool_size: env::var("GRPC_ACTOR_POOL_SIZE")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),
            grpc_cors_origins: env::var("GRPC_CORS_ORIGINS")
                .unwrap_or_else(|_| "*".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),

            // Redis Configuration
            redis_url: env::var("REDIS_URL")
                .map_err(|_| ConfigError::MissingEnvVar("REDIS_URL".to_string()))?,
            redis_pool_size: env::var("REDIS_POOL_SIZE")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .unwrap_or(5),

            // NATS Configuration
            nats_url: env::var("NATS_URL")
                .map_err(|_| ConfigError::MissingEnvVar("NATS_URL".to_string()))?,
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

    #[error("ScyllaDB configuration error: {0}")]
    ScyllaConfig(String),
}
