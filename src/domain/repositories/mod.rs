/// Repository trait definitions (interfaces)
///
/// These traits define the contracts for data access operations.
/// Implementations are provided in the infrastructure layer.
pub mod auth;
pub mod post;
pub mod user;

// Re-export repository traits
pub use auth::{AuthRepository, AuthRepositoryError};
pub use post::{PostRepository, PostRepositoryError};
pub use user::UserRepository;

// Backward compatibility (deprecated)
#[deprecated(since = "0.3.0", note = "Use `auth` module instead")]
pub use auth as auth_repository;

#[deprecated(since = "0.3.0", note = "Use `user` module instead")]
pub use user as user_repository;

#[deprecated(since = "0.3.0", note = "Use `post` module instead")]
pub use post as post_repository;
