use crate::{
    domain::{entities::User, repositories::user_repository::UserRepository},
    shared::AppError,
};
use std::sync::Arc;

/// Query for listing users with pagination (Read operation - optimized)
pub struct ListUsersQuery<R: UserRepository> {
    user_repository: Arc<R>,
}

impl<R: UserRepository> ListUsersQuery<R> {
    pub fn new(user_repository: Arc<R>) -> Self {
        Self { user_repository }
    }

    pub async fn execute(&self, page: i64, page_size: i64) -> Result<(Vec<User>, i64), AppError> {
        // Validate pagination parameters
        if page < 1 {
            return Err(AppError::Validation("Page must be >= 1".to_string()));
        }

        if !(1..=100).contains(&page_size) {
            return Err(AppError::Validation("Page size must be between 1 and 100".to_string()));
        }

        // Future: Add caching for frequently accessed pages
        // let cache_key = format!("users:page:{}:size:{}", page, page_size);
        // if let Some(cached_result) = cache.get(&cache_key).await {
        //     return Ok(cached_result);
        // }

        let users = self.user_repository.list_paginated(page, page_size).await?;

        // Get total count for pagination metadata
        let total = self.user_repository.count().await?;

        // Future: Cache the result
        // cache.set(&cache_key, &(users.clone(), total), Duration::from_secs(60)).await;

        tracing::debug!("Listed {} users (page {}, total {})", users.len(), page, total);

        Ok((users, total))
    }

    /// Get users with filtering and sorting (advanced query)
    pub async fn execute_with_filters(
        &self,
        page: i64,
        page_size: i64,
        _filters: UserFilters,
    ) -> Result<(Vec<User>, i64), AppError> {
        // Future: Implement filtering logic
        // For now, just call the basic list
        self.execute(page, page_size).await
    }
}

/// Filters for user queries
#[derive(Debug, Clone, Default)]
pub struct UserFilters {
    pub email_contains: Option<String>,
    pub name_contains: Option<String>,
    pub is_active: Option<bool>,
    pub role: Option<String>,
    pub created_after: Option<chrono::DateTime<chrono::Utc>>,
    pub created_before: Option<chrono::DateTime<chrono::Utc>>,
}
