 use crate::infrastructure::cache::CacheRepository;
use crate::{
    application::dto::UpdateUserDto,
    domain::{
        entities::User,
        repositories::user_repository::UserRepository, value_objects::UserId,
    },
    infrastructure::messaging::{
        MessagingService,
        NatsClient, 
        events::{v2::UserUpdatedEventV2, traits::Event},
        subjects::{SubjectVersion, UserEventType, UserSubject},
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
    nats_client: Arc<NatsClient>,
}

impl<R: UserRepository, C: CacheRepository + ?Sized> UpdateUserUseCase<R, C> {
    pub fn new(
        user_repository: Arc<R>,
        cache_repository: Arc<C>,
        nats_client: Arc<NatsClient>,
    ) -> Self {
        Self {
            user_repository,
            cache_repository,
            nats_client,
        }
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
            .find_by_id(user_id.clone())
            .await?
            .ok_or_else(|| AppError::NotFound(format!("User with ID {} not found", user_id)))?;

        // Capture previous state for event
        let previous_name = user.name.clone();

        // Update name if provided
        if let Some(name) = dto.name.clone() {
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

        // Publish UserUpdatedEventV2 to NATS
        self.publish_user_updated_event(&updated_user, previous_name).await;

        tracing::info!("User updated successfully: {}", updated_user.id);

        Ok(updated_user)
    }

    /// Publish user updated event to NATS
    async fn publish_user_updated_event(&self, user: &User, previous_name: String) {
        // Build the event with field changes
        let mut event = UserUpdatedEventV2::new(user.id.to_string());

        // Track name change if it occurred
        if previous_name != user.name {
            event = event.with_name_change(
                Some(previous_name),
                Some(user.name.clone()),
            );
        }

        // Serialize event to bytes
        let payload = match event.to_bytes() {
            Ok(bytes) => bytes,
            Err(e) => {
                tracing::error!("Failed to serialize UserUpdatedEventV2: {}", e);
                return;
            }
        };

        // Get environment name from env var or default to "dev"
        let env = std::env::var("APP_ENV").unwrap_or_else(|_| "dev".to_string());

        // Build NATS subject: {env}.users.v2.updated
        let subject = UserSubject::build(&env, SubjectVersion::V2, UserEventType::Updated);

        // Publish to NATS
        if let Err(e) = self.nats_client.publish(subject.clone(), payload).await {
            tracing::warn!("Failed to publish UserUpdatedEventV2 to {}: {}", subject, e);
        } else {
            tracing::info!("Published UserUpdatedEventV2 to {}", subject);
        }
    }
}
