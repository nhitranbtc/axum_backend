/// Database repository implementations
///
/// This module contains concrete implementations of repository traits.
/// Implementations are organized by domain (user, auth, etc.) rather than
/// by database technology to avoid coupling.
pub mod auth;
pub mod user;

// Re-export with descriptive names
pub use auth::RepositoryImpl as AuthRepositoryImpl;
pub use user::RepositoryImpl as UserRepositoryImpl;

// Backward compatibility (deprecated)
#[deprecated(since = "0.2.0", note = "Use `UserRepositoryImpl` instead")]
pub use user::RepositoryImpl as PostgresUserRepository;

#[deprecated(since = "0.2.0", note = "Use `AuthRepositoryImpl` instead")]
pub use auth::RepositoryImpl as PostgresAuthRepository;
