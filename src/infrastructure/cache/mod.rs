pub mod distributed_lock;
pub mod rate_limiter;
pub mod redis_cache;
use async_trait::async_trait;
pub use distributed_lock::DistributedLock;
pub use rate_limiter::RateLimiter;
pub use redis_cache::RedisCacheRepository;
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CacheError {
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Connection error: {0}")]
    Connection(String),
}

#[async_trait]
pub trait CacheRepository: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<String>, CacheError>;
    async fn set(&self, key: &str, value: &str, ttl: Duration) -> Result<(), CacheError>;
    async fn delete(&self, key: &str) -> Result<(), CacheError>;
    async fn set_nx(&self, key: &str, value: &str, ttl: Duration) -> Result<bool, CacheError>;
    async fn delete_if_equals(&self, key: &str, value: &str) -> Result<bool, CacheError>;
}
