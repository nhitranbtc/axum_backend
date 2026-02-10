use crate::{
    domain::{repositories::AuthRepository, value_objects::Email},
    shared::utils::password::PasswordManager,
};
use std::sync::Arc;

#[derive(Debug, thiserror::Error)]
pub enum SetPasswordError {
    #[error("User not found")]
    UserNotFound,

    #[error("Invalid email format")]
    InvalidEmail,

    #[error("Invalid confirmation code")]
    InvalidCode,

    #[error("Confirmation code expired")]
    CodeExpired,

    #[error("Password hashing failed: {0}")]
    PasswordHashError(String),

    #[error("Repository error: {0}")]
    RepositoryError(String),
}

pub struct SetPasswordUseCase<R: AuthRepository> {
    auth_repo: Arc<R>,
}

impl<R: AuthRepository> SetPasswordUseCase<R> {
    pub fn new(auth_repo: Arc<R>) -> Self {
        Self { auth_repo }
    }

    pub async fn execute(
        &self,
        email: String,
        code: String,
        new_password: String,
    ) -> Result<String, SetPasswordError> {
        let email_vo = Email::parse(&email).map_err(|_| SetPasswordError::InvalidEmail)?;

        let mut user = self
            .auth_repo
            .find_by_email(email_vo.as_str())
            .await
            .map_err(|e| SetPasswordError::RepositoryError(e.to_string()))?
            .ok_or(SetPasswordError::UserNotFound)?;

        // Check code
        match &user.confirmation_code {
            Some(c) if c == &code => {
                if let Some(expires_at) = user.confirmation_code_expires_at {
                    if chrono::Utc::now() > expires_at {
                        return Err(SetPasswordError::CodeExpired);
                    }
                } else {
                    return Err(SetPasswordError::InvalidCode);
                }
            },
            _ => return Err(SetPasswordError::InvalidCode),
        }

        // Hash password
        let password_hash =
            tokio::task::spawn_blocking(move || PasswordManager::hash(&new_password))
                .await
                .map_err(|e| SetPasswordError::PasswordHashError(e.to_string()))?
                .map_err(|e| SetPasswordError::PasswordHashError(e.to_string()))?;

        // Set password and clear code (now we can clear it, as password is set)
        user.set_password(password_hash);
        user.confirmation_code = None;
        user.confirmation_code_expires_at = None;
        user.is_active = true;
        user.is_email_verified = true;

        self.auth_repo
            .update_user(&user)
            .await
            .map_err(|e| SetPasswordError::RepositoryError(e.to_string()))?;

        Ok("Password set successfully.".to_string())
    }
}
