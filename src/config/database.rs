use deadpool::Runtime;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::AsyncPgConnection;
use std::env;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub max_connections: usize,
    pub min_connections: usize,
    pub connect_timeout: Duration,
    pub idle_timeout: Duration,
    pub max_lifetime: Duration,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            max_connections: 10,
            min_connections: 2,
            connect_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(600),
            max_lifetime: Duration::from_secs(1800),
        }
    }
}

impl DatabaseConfig {
    pub fn from_env() -> Self {
        Self {
            max_connections: env::var("DB_MAX_CONNECTIONS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),
            min_connections: env::var("DB_MIN_CONNECTIONS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(2),
            connect_timeout: env::var("DB_CONNECT_TIMEOUT_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .map(Duration::from_secs)
                .unwrap_or(Duration::from_secs(30)),
            idle_timeout: env::var("DB_IDLE_TIMEOUT_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .map(Duration::from_secs)
                .unwrap_or(Duration::from_secs(600)),
            max_lifetime: env::var("DB_MAX_LIFETIME_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .map(Duration::from_secs)
                .unwrap_or(Duration::from_secs(1800)),
        }
    }

    pub fn create_pool(&self, database_url: &str) -> Pool<AsyncPgConnection> {
        let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);

        Pool::builder(config)
            .max_size(self.max_connections)
            .wait_timeout(Some(self.connect_timeout))
            .create_timeout(Some(self.connect_timeout))
            .recycle_timeout(Some(self.idle_timeout))
            .runtime(Runtime::Tokio1)
            .build()
            .expect("Failed to create database pool")
    }
}
