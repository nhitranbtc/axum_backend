use crate::domain::{repositories::AuthRepository, value_objects::Email};
use std::sync::Arc;
use tracing::error;

#[derive(Debug, thiserror::Error)]
pub enum VerifyEmailError {
    #[error("User not found")]
    UserNotFound,

    #[error("Invalid email format")]
    InvalidEmail,

    #[error("Invalid confirmation code")]
    InvalidCode,

    #[error("Confirmation code expired")]
    CodeExpired,

    #[error("Repository error: {0}")]
    RepositoryError(String),
}

pub struct VerifyEmailUseCase<R: AuthRepository> {
    auth_repo: Arc<R>,
}

impl<R: AuthRepository> VerifyEmailUseCase<R> {
    pub fn new(auth_repo: Arc<R>) -> Self {
        Self { auth_repo }
    }

    pub async fn execute(&self, email: String, code: String) -> Result<String, VerifyEmailError> {
        let email_vo = Email::parse(&email).map_err(|_| VerifyEmailError::InvalidEmail)?;

        let mut user = self
            .auth_repo
            .find_by_email(email_vo.as_str())
            .await
            .map_err(|e| VerifyEmailError::RepositoryError(e.to_string()))?
            .ok_or(VerifyEmailError::UserNotFound)?;

        // Check if code matches
        match &user.confirmation_code {
            Some(c) if c == &code => {
                // Check expiry
                if let Some(expires_at) = user.confirmation_code_expires_at {
                    if chrono::Utc::now() > expires_at {
                        return Err(VerifyEmailError::CodeExpired);
                    }
                } else {
                    // Should not happen if code is present
                    return Err(VerifyEmailError::InvalidCode);
                }
            },
            _ => return Err(VerifyEmailError::InvalidCode),
        }

        // Verify user
        user.verify_email();

        self.auth_repo
            .update_user(&user)
            .await
            .map_err(|e| VerifyEmailError::RepositoryError(e.to_string()))?;

        Ok("Email verified successfully.".to_string())
    }
}
