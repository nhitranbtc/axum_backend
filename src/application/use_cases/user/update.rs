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
pub struct UpdateUserUseCase<R: UserRepository> {
    user_repository: Arc<R>,
}

impl<R: UserRepository> UpdateUserUseCase<R> {
    pub fn new(user_repository: Arc<R>) -> Self {
        Self { user_repository }
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

        tracing::info!("User updated successfully: {}", updated_user.id);

        Ok(updated_user)
    }
}
