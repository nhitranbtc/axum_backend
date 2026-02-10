/// User domain use cases
///
/// Use cases orchestrate business logic for user-related operations.
/// Each use case represents a single business operation.
pub mod create;
pub mod get;
pub mod import;
pub mod list;
pub mod roles;
pub mod update;

// Re-export use case types
pub use create::CreateUserUseCase;
pub use get::GetUserUseCase;
pub use import::ImportUsersUseCase;
pub use list::ListUsersUseCase;
pub use roles::{GetUserRoleUseCase, UpdateUserRoleUseCase};
pub use update::UpdateUserUseCase;

// Backward compatibility (deprecated)
#[deprecated(since = "0.3.0", note = "Use `create` module instead")]
pub use create as create_user;

#[deprecated(since = "0.3.0", note = "Use `get` module instead")]
pub use get as get_user;

#[deprecated(since = "0.3.0", note = "Use `import` module instead")]
pub use import as import_users;

#[deprecated(since = "0.3.0", note = "Use `list` module instead")]
pub use list as list_users;

#[deprecated(since = "0.3.0", note = "Use `roles` module instead")]
pub use roles as role_management;

#[deprecated(since = "0.3.0", note = "Use `update` module instead")]
pub use update as update_user;
