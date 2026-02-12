use crate::{
    application::{
        dto::auth::{RegisterResponse, UserInfo},
        services::email::{EmailService, EmailType, Recipient},
    },
    domain::{
        repositories::{AuthRepository, AuthRepositoryError},
        value_objects::Email,
    },
    infrastructure::cache::{CacheRepository, DistributedLock},
};
use std::{sync::Arc, time::Duration};
use tracing::{error, info};

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

    #[error("Concurrent registration attempt")]
    ConcurrentRegistration,

    #[error("Lock error: {0}")]
    LockError(String),
}

pub struct RegisterUseCase<R: AuthRepository, C: CacheRepository + ?Sized> {
    auth_repo: Arc<R>,
    email_service: Arc<dyn EmailService>,
    cache_repository: Arc<C>,
    confirm_code_expiry: i64,
}

impl<R: AuthRepository, C: CacheRepository + ?Sized> RegisterUseCase<R, C> {
    pub fn new(
        auth_repo: Arc<R>,
        email_service: Arc<dyn EmailService>,
        cache_repository: Arc<C>,
        confirm_code_expiry: i64,
    ) -> Self {
        Self { auth_repo, email_service, cache_repository, confirm_code_expiry }
    }

    pub async fn execute(
        &self,
        email: String,
        name: String,
    ) -> Result<RegisterResponse, RegisterError> {
        // Validate email format
        let email_vo = Email::parse(&email).map_err(|_| RegisterError::InvalidEmail)?;

        // Distributed Lock to prevent concurrent registration
        let lock_key = format!("lock:register:{}", email_vo.as_str());
        let lock_value = uuid::Uuid::new_v4().to_string();
        let lock_ttl = Duration::from_secs(10); // Hold lock for 10 seconds max

        let lock =
            DistributedLock::new(self.cache_repository.clone(), lock_key, lock_value, lock_ttl);

        if !lock.acquire().await.map_err(|e| RegisterError::LockError(e.to_string()))? {
            return Err(RegisterError::ConcurrentRegistration);
        }

        // Use a closure or explicit release to ensure lock is released (RAII not fully implemented for async)
        // We will manually release at end.

        // Check if user already exists
        if (self
            .auth_repo
            .find_by_email(email_vo.as_str())
            .await
            .map_err(|e| RegisterError::RepositoryError(e.to_string()))?)
        .is_some()
        {
            let _ = lock.release().await; // Release lock before returning
            return Err(RegisterError::EmailAlreadyExists);
        }

        // Generate Confirmation Code
        use rand::Rng;
        let confirmation_code: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Uniform::new(0, 10))
            .take(6)
            .map(|x| x.to_string())
            .collect();

        let expires_at = chrono::Utc::now() + chrono::Duration::seconds(self.confirm_code_expiry);

        // Create user (inactive, no password)
        let user_result = self
            .auth_repo
            .create_user(
                email_vo.as_str(),
                &name,
                None, // No password
                Some(confirmation_code.clone()),
                Some(expires_at),
            )
            .await;

        let user = match user_result {
            Ok(u) => u,
            Err(e) => {
                let _ = lock.release().await;
                return Err(match e {
                    AuthRepositoryError::EmailAlreadyExists => RegisterError::EmailAlreadyExists,
                    _ => RegisterError::RepositoryError(e.to_string()),
                });
            },
        };

        // Send confirmation email
        let recipient = Recipient { email: email_vo.as_str().to_string(), name: user.name.clone() };

        if let Err(e) = self
            .email_service
            .send(recipient, EmailType::Confirmation(confirmation_code))
            .await
        {
            error!("Failed to send confirmation email: {}", e);
            let _ = lock.release().await;
            return Err(RegisterError::EmailError(e.to_string()));
        }

        // Release Lock
        if let Err(e) = lock.release().await {
            // Log but don't fail the request as operation succeeded
            error!("Failed to release register lock: {}", e);
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
