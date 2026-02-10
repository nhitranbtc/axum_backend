use crate::{
    domain::{
        entities::User, repositories::user_repository::UserRepository, value_objects::UserId,
    },
    shared::AppError,
};
use std::sync::Arc;
use uuid::Uuid;

/// Use case for getting a user by ID
pub struct GetUserUseCase<R: UserRepository> {
    user_repository: Arc<R>,
}

impl<R: UserRepository> GetUserUseCase<R> {
    pub fn new(user_repository: Arc<R>) -> Self {
        Self { user_repository }
    }

    pub async fn execute(&self, user_id: &str) -> Result<User, AppError> {
        // Parse UUID
        let uuid = Uuid::parse_str(user_id)
            .map_err(|_| AppError::Validation("Invalid user ID format".to_string()))?;

        let user_id = UserId::from_uuid(uuid);

        // Find user
        let user = self
            .user_repository
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("User with ID {} not found", user_id)))?;

        Ok(user)
    }
}
