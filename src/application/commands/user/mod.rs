/// User commands (write operations)
///
/// Commands represent write operations that modify state.
/// Each command is responsible for validating input and coordinating
/// with the domain layer to execute business logic.
pub mod create;
pub mod update;

// Re-export command types
pub use create::CreateUserCommand;
pub use update::UpdateUserCommand;

// Backward compatibility (deprecated)
#[deprecated(since = "0.3.0", note = "Use `create` module instead")]
pub use create as create_user_command;

#[deprecated(since = "0.3.0", note = "Use `update` module instead")]
pub use update as update_user_command;
