use crate::{
    domain::{
        entities::User,
        repositories::user_repository::{RepositoryError, UserRepository},
        value_objects::{Email, UserId},
    },
    shared::AppError,
};
use std::sync::Arc;

/// Service for user-related business logic that spans multiple use cases
pub struct UserService<R: UserRepository> {
    user_repository: Arc<R>,
}

impl<R: UserRepository> UserService<R> {
    pub fn new(user_repository: Arc<R>) -> Self {
        Self { user_repository }
    }

    /// Check if a user exists by email
    pub async fn user_exists_by_email(&self, email: &Email) -> Result<bool, AppError> {
        self.user_repository.exists_by_email(email).await.map_err(|e| e.into())
    }

    /// Get user by ID with caching support (future enhancement)
    pub async fn get_user_by_id(&self, user_id: &UserId) -> Result<Option<User>, AppError> {
        self.user_repository.find_by_id(*user_id).await.map_err(|e| e.into())
    }

    /// Get user by email
    pub async fn get_user_by_email(&self, email: &Email) -> Result<Option<User>, AppError> {
        self.user_repository.find_by_email(email).await.map_err(|e| e.into())
    }

    /// Validate user can be deleted (business rules)
    pub async fn can_delete_user(&self, user_id: &UserId) -> Result<bool, AppError> {
        // Add business logic here
        // Example: Check if user has active sessions, pending orders, etc.
        let user = self.get_user_by_id(user_id).await?;

        match user {
            Some(_) => {
                // Future: Add more complex validation
                // - Check for active sessions
                // - Check for ownership of resources
                // - Check for pending transactions
                Ok(true)
            },
            None => Ok(false),
        }
    }

    /// Get user statistics (for analytics)
    pub async fn get_user_count(&self) -> Result<i64, AppError> {
        // This would be implemented in the repository
        // For now, we'll return a placeholder
        Ok(0)
    }
}

impl From<RepositoryError> for AppError {
    fn from(err: RepositoryError) -> Self {
        match err {
            RepositoryError::NotFound => AppError::NotFound("Resource not found".to_string()),
            RepositoryError::DuplicateEmail(msg) => AppError::Validation(msg),
            RepositoryError::Database(msg) => {
                tracing::error!("Database error: {}", msg);
                AppError::Internal(anyhow::anyhow!("Database error"))
            },
            RepositoryError::Internal(msg) => {
                tracing::error!("Repository internal error: {}", msg);
                AppError::Internal(anyhow::anyhow!("Internal error"))
            },
        }
    }
}
