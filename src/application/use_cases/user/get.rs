use crate::{
    domain::{
        entities::User, repositories::user_repository::UserRepository, value_objects::UserId,
    },
    infrastructure::cache::CacheRepository,
    shared::AppError,
};
use std::{sync::Arc, time::Duration};
use tracing::{debug, error};
use uuid::Uuid;

/// Use case for getting a user by ID
pub struct GetUserUseCase<R: UserRepository, C: CacheRepository + ?Sized> {
    user_repository: Arc<R>,
    cache_repository: Arc<C>,
}

impl<R: UserRepository, C: CacheRepository + ?Sized> GetUserUseCase<R, C> {
    pub fn new(user_repository: Arc<R>, cache_repository: Arc<C>) -> Self {
        Self { user_repository, cache_repository }
    }

    pub async fn execute(&self, user_id: &str) -> Result<User, AppError> {
        // Parse UUID
        let uuid = Uuid::parse_str(user_id)
            .map_err(|_| AppError::Validation("Invalid user ID format".to_string()))?;

        let user_id = UserId::from_uuid(uuid);
        let cache_key = format!("user:{}", user_id);

        // 1. Check Cache
        match self.cache_repository.get(&cache_key).await {
            Ok(Some(cached_json)) => match serde_json::from_str::<User>(&cached_json) {
                Ok(user) => {
                    debug!("Cache HIT for user {}", user_id);
                    return Ok(user);
                },
                Err(e) => error!("Failed to deserialize cached user {}: {}", user_id, e),
            },
            Ok(None) => debug!("Cache MISS for user {}", user_id),
            Err(e) => error!("Cache error for user {}: {}", user_id, e),
        }

        // 2. Fetch from DB
        let user = self
            .user_repository
            .find_by_id(user_id.clone())
            .await?
            .ok_or_else(|| AppError::NotFound(format!("User with ID {} not found", user_id)))?;

        // 3. Update Cache (Async, ignore errors)
        match serde_json::to_string(&user) {
            Ok(json) => {
                // TTL: 10 minutes
                if let Err(e) =
                    self.cache_repository.set(&cache_key, &json, Duration::from_secs(600)).await
                {
                    error!("Failed to cache user {}: {}", user_id, e);
                } else {
                    debug!("Cached user {}", user_id);
                }
            },
            Err(e) => error!("Failed to serialize user {} for cache: {}", user_id, e),
        }

        Ok(user)
    }
}
