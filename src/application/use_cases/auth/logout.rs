use crate::domain::repositories::{AuthRepository, AuthRepositoryError};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum LogoutError {
    #[error("Repository error: {0}")]
    RepositoryError(String),

    #[error("Token not found")]
    TokenNotFound,
}

pub struct LogoutUseCase<R: AuthRepository> {
    auth_repo: Arc<R>,
}

impl<R: AuthRepository> LogoutUseCase<R> {
    pub fn new(auth_repo: Arc<R>) -> Self {
        Self { auth_repo }
    }

    /// Logout from current session (revoke specific refresh token)
    pub async fn execute(&self, refresh_token: &str) -> Result<(), LogoutError> {
        self.auth_repo.revoke_refresh_token(refresh_token).await.map_err(|e| match e {
            AuthRepositoryError::TokenNotFound => LogoutError::TokenNotFound,
            _ => LogoutError::RepositoryError(e.to_string()),
        })
    }

    /// Logout from all sessions (revoke all user's refresh tokens)
    pub async fn execute_all(&self, user_id: Uuid) -> Result<(), LogoutError> {
        self.auth_repo
            .revoke_all_user_tokens(user_id)
            .await
            .map_err(|e| LogoutError::RepositoryError(e.to_string()))
    }
}
