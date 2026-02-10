/// Business logic services
///
/// Services encapsulate complex business logic that spans multiple use cases
/// or requires coordination between different domain entities.
pub mod auth;
pub mod email;
pub mod user;

// Re-export for convenience
pub use auth::AuthService;
pub use user::UserService;

// Backward compatibility (deprecated)
#[deprecated(since = "0.3.0", note = "Use `auth` module instead")]
pub use auth as auth_service;

#[deprecated(since = "0.3.0", note = "Use `user` module instead")]
pub use user as user_service;
