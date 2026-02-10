/// Actors for background processing and async operations
///
/// Actors handle long-running or background tasks that don't fit
/// into the request-response cycle.
pub mod import;

// Re-export actor types
pub use import::{UserCreationActor, UserCreationMsg};

// Backward compatibility (deprecated)
#[deprecated(since = "0.3.0", note = "Use `import` module instead")]
pub use import as user_import_actor;
