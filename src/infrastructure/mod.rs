pub mod cache;
pub mod database;
pub mod email;
pub mod external_apis;
pub mod monitoring;

// Re-export commonly used items
pub use cache::redis_cache::RedisCacheRepository;
pub use cache::CacheRepository;
pub use database::repositories::{AuthRepositoryImpl, UserRepositoryImpl};
pub use monitoring::SystemMonitor;
