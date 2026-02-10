/// Data Transfer Objects (DTOs) for API communication
///
/// DTOs define the structure of data sent to and from the API.
/// Organized by domain for better maintainability.
pub mod auth;
pub mod role;
pub mod user;

// Re-export commonly used DTOs
pub use auth::*;
pub use role::*;
pub use user::*;

// Backward compatibility (deprecated)
#[deprecated(since = "0.3.0", note = "Use `auth` module instead")]
pub use auth as auth_dto;

#[deprecated(since = "0.3.0", note = "Use `user` module instead")]
pub use user as user_dto;

#[deprecated(since = "0.3.0", note = "Use `role` module instead")]
pub use role as role_dto;
