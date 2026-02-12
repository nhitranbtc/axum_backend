use crate::infrastructure::cache::CacheRepository;
use crate::{
    application::dto::UpdateUserDto,
    domain::{
        entities::User, repositories::user_repository::UserRepository, value_objects::UserId,
    },
    shared::AppError,
};
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

/// Use case for updating a user
pub struct UpdateUserUseCase<R: UserRepository, C: CacheRepository + ?Sized> {
    user_repository: Arc<R>,
    cache_repository: Arc<C>,
}

impl<R: UserRepository, C: CacheRepository + ?Sized> UpdateUserUseCase<R, C> {
    pub fn new(user_repository: Arc<R>, cache_repository: Arc<C>) -> Self {
        Self { user_repository, cache_repository }
    }

    pub async fn execute(&self, user_id: &str, dto: UpdateUserDto) -> Result<User, AppError> {
        // Validate input
        dto.validate().map_err(|e| AppError::Validation(e.to_string()))?;

        // Parse UUID
        let uuid = Uuid::parse_str(user_id)
            .map_err(|_| AppError::Validation("Invalid user ID format".to_string()))?;

        let user_id = UserId::from_uuid(uuid);

        // Find existing user
        let mut user = self
            .user_repository
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("User with ID {} not found", user_id)))?;

        // Update name if provided
        if let Some(name) = dto.name {
            user.update_name(name).map_err(|e| AppError::Validation(e.to_string()))?;
        }

        // Save updated user
        let updated_user = self.user_repository.save(&user).await?;

        // Invalidate cache
        let cache_key = format!("user:{}", user_id);
        if let Err(e) = self.cache_repository.delete(&cache_key).await {
            tracing::warn!("Failed to invalidate cache for user {}: {}", user_id, e);
        } else {
            tracing::debug!("Invalidated cache for user {}", user_id);
        }

        tracing::info!("User updated successfully: {}", updated_user.id);

        Ok(updated_user)
    }
}
