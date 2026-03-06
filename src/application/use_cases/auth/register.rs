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
    shared::utils::password::PasswordManager,
};
use std::{sync::Arc, time::Duration};
use tracing::{error, info, warn};

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
        password: Option<String>,
    ) -> Result<RegisterResponse, RegisterError> {
        // Validate email early so all following logic can rely on normalized value.
        let email_vo = Email::parse(&email).map_err(|_| RegisterError::InvalidEmail)?;
        info!(email = %email_vo.as_str(), "starting user registration");

        let lock_key = format!("lock:register:{}", email_vo.as_str());
        let lock_value = uuid::Uuid::new_v4().to_string();
        let lock_ttl = Duration::from_secs(10);

        let lock =
            DistributedLock::new(self.cache_repository.clone(), lock_key, lock_value, lock_ttl);

        if !lock.acquire().await.map_err(|e| RegisterError::LockError(e.to_string()))? {
            info!(email = %email_vo.as_str(), "registration blocked by concurrent lock");
            return Err(RegisterError::ConcurrentRegistration);
        }

        if (self
            .auth_repo
            .find_by_email(email_vo.as_str())
            .await
            .map_err(|e| RegisterError::RepositoryError(e.to_string()))?)
        .is_some()
        {
            if let Err(e) = lock.release().await {
                error!(email = %email_vo.as_str(), error = %e, "failed to release registration lock");
            }
            return Err(RegisterError::EmailAlreadyExists);
        }

        let confirmation_code = generate_confirmation_code();
        let expires_at = chrono::Utc::now() + chrono::Duration::seconds(self.confirm_code_expiry);
        let password_hash = hash_password(password).await?;

        let user_result = self
            .auth_repo
            .create_user(
                email_vo.as_str(),
                &name,
                password_hash,
                Some(confirmation_code.clone()),
                Some(expires_at),
            )
            .await;

        let user = match user_result {
            Ok(u) => u,
            Err(e) => {
                if let Err(release_err) = lock.release().await {
                    error!(
                        email = %email_vo.as_str(),
                        error = %release_err,
                        "failed to release registration lock after repository error"
                    );
                }
                return Err(match e {
                    AuthRepositoryError::EmailAlreadyExists => RegisterError::EmailAlreadyExists,
                    _ => RegisterError::RepositoryError(e.to_string()),
                });
            },
        };

        let recipient = Recipient { email: email_vo.as_str().to_string(), name: user.name.clone() };

        if let Err(e) = self
            .email_service
            .send(recipient, EmailType::Confirmation(confirmation_code))
            .await
        {
            // Registration still succeeds to avoid blocking account creation on transient email failures.
            warn!(
                email = %email_vo.as_str(),
                error = %e,
                "confirmation email delivery failed after user creation"
            );
        }

        if let Err(e) = lock.release().await {
            error!(
                email = %email_vo.as_str(),
                error = %e,
                "failed to release registration lock"
            );
        }

        info!(
            email = %email_vo.as_str(),
            user_id = %user.id.as_uuid(),
            "user registration completed"
        );

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

fn generate_confirmation_code() -> String {
    use rand::Rng;
    rand::thread_rng()
        .sample_iter(&rand::distributions::Uniform::new(0, 10))
        .take(6)
        .map(|x| x.to_string())
        .collect()
}

async fn hash_password(password: Option<String>) -> Result<Option<String>, RegisterError> {
    match password {
        Some(pw) => {
            let hash = tokio::task::spawn_blocking(move || PasswordManager::hash(&pw))
                .await
                .map_err(|e| RegisterError::PasswordHashError(e.to_string()))?
                .map_err(|e| RegisterError::PasswordHashError(e.to_string()))?;
            Ok(Some(hash))
        },
        None => Ok(None),
    }
}
