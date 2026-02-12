use crate::{
    domain::{repositories::user_repository::UserRepository, value_objects::UserId},
    infrastructure::cache::CacheRepository,
    shared::AppError,
};
use std::sync::Arc;
use uuid::Uuid;

/// Use case for deleting a user
pub struct DeleteUserUseCase<R: UserRepository, C: CacheRepository + ?Sized> {
    user_repository: Arc<R>,
    cache_repository: Arc<C>,
}

impl<R: UserRepository, C: CacheRepository + ?Sized> DeleteUserUseCase<R, C> {
    pub fn new(user_repository: Arc<R>, cache_repository: Arc<C>) -> Self {
        Self { user_repository, cache_repository }
    }

    pub async fn execute(&self, user_id: &str) -> Result<(), AppError> {
        // Parse UUID
        let uuid = Uuid::parse_str(user_id)
            .map_err(|_| AppError::Validation("Invalid user ID format".to_string()))?;

        let user_id = UserId::from_uuid(uuid);

        // Check if user exists
        if self.user_repository.find_by_id(user_id.clone()).await?.is_none() {
            return Err(AppError::NotFound(format!("User with ID {} not found", user_id)));
        }

        // Delete user
        self.user_repository.delete(user_id.clone()).await?;

        // Invalidate cache
        let cache_key = format!("user:{}", user_id);
        if let Err(e) = self.cache_repository.delete(&cache_key).await {
            tracing::warn!("Failed to invalidate cache for user {}: {}", user_id, e);
        } else {
            tracing::debug!("Invalidated cache for user {}", user_id);
        }

        tracing::info!("User deleted successfully: {}", user_id);

        Ok(())
    }
}
