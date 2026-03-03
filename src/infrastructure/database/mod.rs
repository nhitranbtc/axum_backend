/// Database infrastructure module
///
/// All persistence is handled through ScyllaDB.
/// The postgres sub-module has been removed; use the `scylla` sub-module for all DB access.

pub mod scylla;

use std::sync::Arc;

// Re-export commonly used items
pub use scylla::{
    create_scylla_session, AuthRepositoryImpl, EventRepository, RefreshTokenRow, ScyllaSession,
    SessionRepository, UserEventRow, UserRepositoryImpl, UserRow, UserSessionRow,
};

// Type alias so upstream code can refer to the session as `DbPool`.
// Using Arc<ScyllaSession> so DbPool::clone() is cheap and repo ::new(pool) calls resolve.
pub type DbPool = Arc<ScyllaSession>;

