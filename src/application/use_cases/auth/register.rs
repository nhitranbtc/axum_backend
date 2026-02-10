use crate::{
    application::{
        dto::auth::{RegisterResponse, UserInfo},
        services::email::{EmailService, EmailType, Recipient},
    },
    domain::{
        repositories::{AuthRepository, AuthRepositoryError},
        value_objects::Email,
    },
};
use std::sync::Arc;
use tracing::error;

#[derive(Debug, thiserror::Error)]
pub enum RegisterError {
    #[error("Email already exists")]
    EmailAlreadyExists,

    #[error("Invalid email format")]
    InvalidEmail,

    #[error("Password hashing failed: {0}")]
    PasswordHashError(String),

    #[error("Repository error: {0}")]
    RepositoryError(String),

    #[error("Token creation failed: {0}")]
    TokenCreationError(String),

    #[error("Failed to send email: {0}")]
    EmailError(String),
}

pub struct RegisterUseCase<R: AuthRepository> {
    auth_repo: Arc<R>,
    email_service: Arc<dyn EmailService>,
    confirm_code_expiry: i64,
}

impl<R: AuthRepository> RegisterUseCase<R> {
    pub fn new(
        auth_repo: Arc<R>,
        email_service: Arc<dyn EmailService>,
        confirm_code_expiry: i64,
    ) -> Self {
        Self { auth_repo, email_service, confirm_code_expiry }
    }

    pub async fn execute(
        &self,
        email: String,
        name: String,
    ) -> Result<RegisterResponse, RegisterError> {
        // Return type might change to simple check?
        // Instructions: "user call register api, in this api, we need send confirm code"
        // It doesn't say we log them in. Usually we return "check your email".
        // BUT existing signature returns AuthResponse.
        // I will change it to return just a message or empty struct,
        // OR I can return AuthResponse with empty tokens if the frontend expects it?
        // No, frontend should expect a different flow.
        // However, to keep modifications minimal on unrelated files (like maybe router expects a type),
        // I should check `src/presentation/routes/auth.rs` later.
        // For now, I'll return a special AuthResponse or change the return type.
        // Let's change return type to Result<(), RegisterError> or Result<String, RegisterError>.
        // But `AuthResponse` is defined in DTO.

        // Validate email format
        let email_vo = Email::parse(&email).map_err(|_| RegisterError::InvalidEmail)?;

        // Check if user already exists
        if (self
            .auth_repo
            .find_by_email(email_vo.as_str())
            .await
            .map_err(|e| RegisterError::RepositoryError(e.to_string()))?)
        .is_some()
        {
            return Err(RegisterError::EmailAlreadyExists);
        }

        // Generate Confirmation Code
        // Simple 6-digit code
        // In production use a proper CSPRNG.
        use rand::Rng;
        let confirmation_code: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Uniform::new(0, 10))
            .take(6)
            .map(|x| x.to_string())
            .collect();

        let expires_at = chrono::Utc::now() + chrono::Duration::seconds(self.confirm_code_expiry);

        // Create user (inactive, no password)
        let user = self
            .auth_repo
            .create_user(
                email_vo.as_str(),
                &name,
                None, // No password
                Some(confirmation_code.clone()),
                Some(expires_at),
            )
            .await
            .map_err(|e| match e {
                AuthRepositoryError::EmailAlreadyExists => RegisterError::EmailAlreadyExists,
                _ => RegisterError::RepositoryError(e.to_string()),
            })?;

        // Send confirmation email
        let recipient = Recipient { email: email_vo.as_str().to_string(), name: user.name.clone() };

        if let Err(e) = self
            .email_service
            .send(recipient, EmailType::Confirmation(confirmation_code))
            .await
        {
            error!("Failed to send confirmation email: {}", e);
            // We return error so client knows retry is needed
            return Err(RegisterError::EmailError(e.to_string()));
        }

        Ok(RegisterResponse {
            message: "Registration successful. Please check your email for the confirmation code."
                .to_string(),
            user: UserInfo {
                id: user.id.as_uuid().to_string(),
                email: user.email.as_str().to_string(),
                name: user.name.clone(),
            },
        })
    }
}
