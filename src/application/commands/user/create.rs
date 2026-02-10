use crate::{
    application::dto::CreateUserDto,
    domain::{entities::User, repositories::user_repository::UserRepository, value_objects::Email},
    shared::AppError,
};
use std::sync::Arc;
use validator::Validate;

/// Command for creating a new user (Write operation)
pub struct CreateUserCommand<R: UserRepository> {
    user_repository: Arc<R>,
}

impl<R: UserRepository> CreateUserCommand<R> {
    pub fn new(user_repository: Arc<R>) -> Self {
        Self { user_repository }
    }

    pub async fn execute(&self, dto: CreateUserDto) -> Result<User, AppError> {
        // Validate input
        dto.validate().map_err(|e| AppError::Validation(e.to_string()))?;

        // Parse email
        let email = Email::parse(&dto.email)
            .map_err(|e| AppError::Validation(format!("Invalid email: {}", e)))?;

        // Check if user already exists
        if self.user_repository.exists_by_email(&email).await? {
            return Err(AppError::Validation(format!(
                "User with email {} already exists",
                dto.email
            )));
        }

        // Create user entity
        let user = User::new(email, dto.name).map_err(|e| AppError::Validation(e.to_string()))?;

        // Save to repository
        let saved_user = self.user_repository.save(&user).await?;

        tracing::info!("User created successfully: {}", saved_user.id);

        Ok(saved_user)
    }
}
