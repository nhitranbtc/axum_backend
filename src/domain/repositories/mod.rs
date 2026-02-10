/// Repository trait definitions (interfaces)
///
/// These traits define the contracts for data access operations.
/// Implementations are provided in the infrastructure layer.
pub mod auth;
pub mod user;

// Re-export repository traits
pub use auth::{AuthRepository, AuthRepositoryError};
pub use user::UserRepository;

// Backward compatibility (deprecated)
#[deprecated(since = "0.3.0", note = "Use `auth` module instead")]
pub use auth as auth_repository;

#[deprecated(since = "0.3.0", note = "Use `user` module instead")]
pub use user as user_repository;
