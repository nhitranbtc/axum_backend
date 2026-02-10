use crate::{
    application::services::email::{EmailService, EmailType, Recipient},
    domain::{repositories::AuthRepository, value_objects::Email},
};
use std::sync::Arc;
use tracing::error;

#[derive(Debug, thiserror::Error)]
pub enum ResendConfirmCodeError {
    #[error("User not found")]
    UserNotFound,

    #[error("Invalid email format")]
    InvalidEmail,

    #[error("User already verified")]
    UserAlreadyVerified,

    #[error("Repository error: {0}")]
    RepositoryError(String),

    #[error("Failed to send email: {0}")]
    EmailError(String),
}

pub struct ResendConfirmCodeUseCase<R: AuthRepository> {
    auth_repo: Arc<R>,
    email_service: Arc<dyn EmailService>,
    confirm_code_expiry: i64,
}

impl<R: AuthRepository> ResendConfirmCodeUseCase<R> {
    pub fn new(
        auth_repo: Arc<R>,
        email_service: Arc<dyn EmailService>,
        confirm_code_expiry: i64,
    ) -> Self {
        Self { auth_repo, email_service, confirm_code_expiry }
    }

    pub async fn execute(&self, email: String) -> Result<String, ResendConfirmCodeError> {
        let email_vo = Email::parse(&email).map_err(|_| ResendConfirmCodeError::InvalidEmail)?;

        // Find user
        let mut user = self
            .auth_repo
            .find_by_email(email_vo.as_str())
            .await
            .map_err(|e| ResendConfirmCodeError::RepositoryError(e.to_string()))?
            .ok_or(ResendConfirmCodeError::UserNotFound)?;

        // Check verification status
        if user.is_email_verified {
            return Err(ResendConfirmCodeError::UserAlreadyVerified);
        }

        // Generate Confirmation Code
        use rand::Rng;
        let confirmation_code: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Uniform::new(0, 10))
            .take(6)
            .map(|x| x.to_string())
            .collect();

        let expires_at = chrono::Utc::now() + chrono::Duration::seconds(self.confirm_code_expiry);

        // Update user
        user.set_confirmation_code(confirmation_code.clone(), expires_at);

        self.auth_repo
            .update_user(&user)
            .await
            .map_err(|e| ResendConfirmCodeError::RepositoryError(e.to_string()))?;

        // Send confirmation email
        let recipient = Recipient { email: email_vo.as_str().to_string(), name: user.name.clone() };

        if let Err(e) = self
            .email_service
            .send(recipient, EmailType::Confirmation(confirmation_code))
            .await
        {
            error!("Failed to send confirmation email: {}", e);
            // Return error so client knows retry is needed
            return Err(ResendConfirmCodeError::EmailError(e.to_string()));
        }

        Ok("Confirmation code resent to your email.".to_string())
    }
}
