pub mod connection;
pub mod models; // New: Organized models by domain
pub mod repositories;
pub mod schema;
pub mod transaction;

// Re-export for convenience
pub use connection::{create_pool, DbPool};
pub use models::{RefreshTokenModel, UserModel};
