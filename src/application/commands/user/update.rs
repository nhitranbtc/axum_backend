use crate::{
    application::dto::UpdateUserDto,
    domain::{
        entities::User, repositories::user_repository::UserRepository, value_objects::UserId,
    },
    shared::AppError,
};
use std::sync::Arc;
use validator::Validate;

/// Command for updating a user (Write operation)
pub struct UpdateUserCommand<R: UserRepository> {
    user_repository: Arc<R>,
}

impl<R: UserRepository> UpdateUserCommand<R> {
    pub fn new(user_repository: Arc<R>) -> Self {
        Self { user_repository }
    }

    pub async fn execute(&self, user_id: UserId, dto: UpdateUserDto) -> Result<User, AppError> {
        // Validate input
        dto.validate().map_err(|e| AppError::Validation(e.to_string()))?;

        // Get existing user
        let mut user = self
            .user_repository
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("User {} not found", user_id)))?;

        // Update fields if provided
        if let Some(name) = dto.name {
            user.update_name(name).map_err(|e| AppError::Validation(e.to_string()))?;
        }

        // Save updated user
        let updated_user = self.user_repository.update(&user).await?;

        tracing::info!("User updated successfully: {}", user_id);

        Ok(updated_user)
    }
}
