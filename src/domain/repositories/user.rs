use crate::domain::{
    entities::User,
    value_objects::{Email, UserId},
};
use async_trait::async_trait;

/// Repository trait for User entity
/// This is defined in the domain layer but implemented in infrastructure
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Save a new user or update existing
    async fn save(&self, user: &User) -> Result<User, RepositoryError>;

    /// Update an existing user
    async fn update(&self, user: &User) -> Result<User, RepositoryError>;

    /// Find user by ID
    async fn find_by_id(&self, id: UserId) -> Result<Option<User>, RepositoryError>;

    /// Find user by email
    async fn find_by_email(&self, email: &Email) -> Result<Option<User>, RepositoryError>;

    /// Check if user exists by email
    async fn exists_by_email(&self, email: &Email) -> Result<bool, RepositoryError>;

    /// Get total count of users
    async fn count(&self) -> Result<i64, RepositoryError>;

    /// List all users with pagination
    async fn list_paginated(&self, limit: i64, offset: i64) -> Result<Vec<User>, RepositoryError>;

    /// Delete user by ID
    async fn delete(&self, id: UserId) -> Result<bool, RepositoryError>;

    /// Delete all users (admin only)
    async fn delete_all(&self) -> Result<usize, RepositoryError>;
}

/// Repository-specific errors
#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("User not found")]
    NotFound,

    #[error("Duplicate email: {0}")]
    DuplicateEmail(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<diesel::result::Error> for RepositoryError {
    fn from(err: diesel::result::Error) -> Self {
        match err {
            diesel::result::Error::NotFound => RepositoryError::NotFound,
            diesel::result::Error::DatabaseError(kind, info) => match kind {
                diesel::result::DatabaseErrorKind::UniqueViolation => {
                    RepositoryError::DuplicateEmail(info.message().to_string())
                },
                _ => RepositoryError::Database(info.message().to_string()),
            },
            _ => RepositoryError::Internal(err.to_string()),
        }
    }
}
