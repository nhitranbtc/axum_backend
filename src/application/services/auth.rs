use crate::{
    domain::{
        entities::RefreshToken, repositories::auth_repository::AuthRepository,
        value_objects::UserId,
    },
    shared::{utils::jwt::JwtManager, AppError},
};
use chrono::{Duration, Utc};
use std::sync::Arc;

/// Service for authentication-related business logic
pub struct AuthService<R: AuthRepository> {
    auth_repository: Arc<R>,
    jwt_manager: Arc<JwtManager>,
}

impl<R: AuthRepository> AuthService<R> {
    pub fn new(auth_repository: Arc<R>, jwt_manager: Arc<JwtManager>) -> Self {
        Self { auth_repository, jwt_manager }
    }

    /// Create access and refresh tokens for a user
    pub fn create_token_pair(&self, user_id: UserId) -> Result<(String, String), AppError> {
        let access_token =
            self.jwt_manager.create_access_token(*user_id.as_uuid()).map_err(|e| {
                AppError::Internal(anyhow::anyhow!("Failed to create access token: {}", e))
            })?;

        let refresh_token =
            self.jwt_manager.create_refresh_token(*user_id.as_uuid()).map_err(|e| {
                AppError::Internal(anyhow::anyhow!("Failed to create refresh token: {}", e))
            })?;

        Ok((access_token, refresh_token))
    }

    /// Store refresh token in database
    pub async fn store_refresh_token(
        &self,
        user_id: UserId,
        token: String,
        expires_in_days: i64,
    ) -> Result<(), AppError> {
        let expires_at = Utc::now() + Duration::days(expires_in_days);

        let refresh_token = RefreshToken {
            id: uuid::Uuid::new_v4(),
            user_id: *user_id.as_uuid(),
            token_hash: token,
            expires_at,
            created_at: Utc::now(),
            revoked_at: None,
        };

        self.auth_repository.save_refresh_token(&refresh_token).await.map_err(|e| {
            AppError::Internal(anyhow::anyhow!("Failed to store refresh token: {}", e))
        })?;

        Ok(())
    }

    /// Verify and validate refresh token
    pub async fn verify_refresh_token(&self, token: &str) -> Result<UserId, AppError> {
        // Verify JWT signature
        let claims = self
            .jwt_manager
            .verify_token(token)
            .map_err(|e| AppError::Unauthorized(format!("Invalid token: {}", e)))?;

        let user_id = UserId::from_string(&claims.sub)
            .map_err(|e| AppError::Unauthorized(format!("Invalid user ID in token: {}", e)))?;

        // Check if token exists in database
        let token_exists =
            self.auth_repository.find_refresh_token(token).await.map_err(|e| {
                AppError::Internal(anyhow::anyhow!("Failed to validate token: {}", e))
            })?;

        if token_exists.is_none() {
            return Err(AppError::Unauthorized("Token has been revoked".to_string()));
        }

        Ok(user_id)
    }

    /// Revoke a specific refresh token
    pub async fn revoke_refresh_token(&self, token: &str) -> Result<(), AppError> {
        self.auth_repository
            .revoke_refresh_token(token)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to revoke token: {}", e)))?;

        Ok(())
    }

    /// Revoke all refresh tokens for a user (logout from all devices)
    pub async fn revoke_all_user_tokens(&self, user_id: &UserId) -> Result<(), AppError> {
        self.auth_repository
            .revoke_all_user_tokens(*user_id.as_uuid())
            .await
            .map_err(|e| {
                AppError::Internal(anyhow::anyhow!("Failed to revoke all tokens: {}", e))
            })?;

        Ok(())
    }

    /// Clean up expired tokens (should be run periodically)
    pub async fn cleanup_expired_tokens(&self) -> Result<u64, AppError> {
        self.auth_repository
            .cleanup_expired_tokens()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to cleanup tokens: {}", e)))
    }
}
