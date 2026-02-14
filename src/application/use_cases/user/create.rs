 use crate::{
    application::dto::CreateUserDto,
    domain::{
        entities::User,
        repositories::user_repository::UserRepository,
        value_objects::Email,
    },
    infrastructure::cache::CacheRepository,
    shared::AppError,
};
use std::sync::Arc;
use validator::Validate;

/// Use case for creating a new user
pub struct CreateUserUseCase<R: UserRepository, C: CacheRepository + ?Sized> {
    user_repository: Arc<R>,
    _cache_repository: Arc<C>, // Kept for potential list invalidation
}

impl<R: UserRepository, C: CacheRepository + ?Sized> CreateUserUseCase<R, C> {
    pub fn new(
        user_repository: Arc<R>,
        _cache_repository: Arc<C>,
    ) -> Self {
        Self {
            user_repository,
            _cache_repository,
        }
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
        // Note: This is a legacy endpoint. For proper authentication, use the /api/auth/register endpoint
        let user = User::new(email.clone(), dto.name.clone())
            .map_err(|e| AppError::Validation(e.to_string()))?;

        // Save to repository
        let saved_user = self.user_repository.save(&user).await?;

        tracing::info!("User created successfully: {}", saved_user.id);

        Ok(saved_user)
    }
}
