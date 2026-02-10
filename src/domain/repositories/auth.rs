use crate::domain::entities::{refresh_token::RefreshToken, user::User};
use async_trait::async_trait;

use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum AuthRepositoryError {
    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("User not found")]
    UserNotFound,

    #[error("Token not found")]
    TokenNotFound,

    #[error("Email already exists")]
    EmailAlreadyExists,
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait AuthRepository: Send + Sync {
    /// Find user by email
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AuthRepositoryError>;

    /// Create a new user with password hash
    async fn create_user(
        &self,
        email: &str,
        name: &str,
        password_hash: Option<String>,
        confirmation_code: Option<String>,
        expires_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<User, AuthRepositoryError>;

    /// Update user's last login timestamp
    async fn update_last_login(&self, user_id: Uuid) -> Result<(), AuthRepositoryError>;

    /// Update user entity (generic update)
    async fn update_user(&self, user: &User) -> Result<User, AuthRepositoryError>;

    /// Save refresh token
    async fn save_refresh_token(&self, token: &RefreshToken) -> Result<(), AuthRepositoryError>;

    /// Find refresh token by token hash
    async fn find_refresh_token(
        &self,
        token_hash: &str,
    ) -> Result<Option<RefreshToken>, AuthRepositoryError>;

    /// Revoke refresh token
    async fn revoke_refresh_token(&self, token_hash: &str) -> Result<(), AuthRepositoryError>;

    /// Revoke all user's refresh tokens (logout from all devices)
    async fn revoke_all_user_tokens(&self, user_id: Uuid) -> Result<(), AuthRepositoryError>;

    /// Clean up expired tokens
    async fn cleanup_expired_tokens(&self) -> Result<u64, AuthRepositoryError>;
}
