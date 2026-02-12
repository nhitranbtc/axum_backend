use crate::{
    domain::{entities::User, repositories::user_repository::UserRepository},
    infrastructure::cache::CacheRepository,
    shared::AppError,
};
use std::{sync::Arc, time::Duration};
use tracing::{debug, error};

/// Use case for listing users with pagination
pub struct ListUsersUseCase<R: UserRepository, C: CacheRepository + ?Sized> {
    user_repository: Arc<R>,
    cache_repository: Arc<C>,
}

impl<R: UserRepository, C: CacheRepository + ?Sized> ListUsersUseCase<R, C> {
    pub fn new(user_repository: Arc<R>, cache_repository: Arc<C>) -> Self {
        Self { user_repository, cache_repository }
    }

    pub async fn execute(&self, page: i64, page_size: i64) -> Result<Vec<User>, AppError> {
        // Validate pagination parameters
        if page < 1 {
            return Err(AppError::Validation("Page must be >= 1".to_string()));
        }

        if !(1..=100).contains(&page_size) {
            return Err(AppError::Validation("Page size must be between 1 and 100".to_string()));
        }

        let cache_key = format!("users:list:page:{}:size:{}", page, page_size);

        // 1. Check Cache
        match self.cache_repository.get(&cache_key).await {
            Ok(Some(cached_json)) => match serde_json::from_str::<Vec<User>>(&cached_json) {
                Ok(users) => {
                    debug!("Cache HIT for user list (page {}, size {})", page, page_size);
                    return Ok(users);
                },
                Err(e) => error!("Failed to deserialize cached user list: {}", e),
            },
            Ok(None) => debug!("Cache MISS for user list (page {}, size {})", page, page_size),
            Err(e) => error!("Cache error for user list: {}", e),
        }

        let offset = (page - 1) * page_size;

        // 2. Fetch users from DB
        let users = self.user_repository.list_paginated(page_size, offset).await?;
        tracing::info!("Listed {} users (page {})", users.len(), page);

        // 3. Update Cache
        // Short TTL for lists (e.g., 60 seconds) to balance freshness and performance
        match serde_json::to_string(&users) {
            Ok(json) => {
                if let Err(e) =
                    self.cache_repository.set(&cache_key, &json, Duration::from_secs(60)).await
                {
                    error!("Failed to cache user list: {}", e);
                } else {
                    debug!("Cached user list (page {}, size {})", page, page_size);
                }
            },
            Err(e) => error!("Failed to serialize user list for cache: {}", e),
        }

        Ok(users)
    }
}
