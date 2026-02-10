use crate::{
    domain::{entities::User, repositories::user_repository::UserRepository},
    shared::AppError,
};
use std::sync::Arc;

/// Use case for listing users with pagination
pub struct ListUsersUseCase<R: UserRepository> {
    user_repository: Arc<R>,
}

impl<R: UserRepository> ListUsersUseCase<R> {
    pub fn new(user_repository: Arc<R>) -> Self {
        Self { user_repository }
    }

    pub async fn execute(&self, page: i64, page_size: i64) -> Result<Vec<User>, AppError> {
        // Validate pagination parameters
        if page < 1 {
            return Err(AppError::Validation("Page must be >= 1".to_string()));
        }

        if !(1..=100).contains(&page_size) {
            return Err(AppError::Validation("Page size must be between 1 and 100".to_string()));
        }

        let offset = (page - 1) * page_size;

        // Fetch users
        let users = self.user_repository.list_paginated(page_size, offset).await?;
        tracing::info!("Listed {} users (page {})", users.len(), page);

        Ok(users)
    }
}
