/// HTTP request handlers organized by domain
///
/// Each module contains handlers for a specific domain area.
/// Handlers are responsible for:
/// - Parsing HTTP requests
/// - Calling use cases/services
/// - Formatting HTTP responses
pub mod auth;
pub mod monitoring;
pub mod role;
pub mod user;

// Re-export handler functions for convenience
pub use role::{get_user_role, update_user_role};
pub use user::{create_user, get_user, import_users, list_users, update_user};

// Backward compatibility (deprecated)
#[deprecated(since = "0.3.0", note = "Use `auth` module instead")]
pub use auth as auth_handlers;

#[deprecated(since = "0.3.0", note = "Use `user` module instead")]
pub use user as user_handlers;

#[deprecated(since = "0.3.0", note = "Use `role` module instead")]
pub use role as role_handlers;
