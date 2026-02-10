use crate::{
    application::dto::auth::{AuthResponse, UserInfo},
    domain::{entities::RefreshToken, repositories::AuthRepository},
    shared::utils::{jwt::JwtManager, password::PasswordManager},
};

use std::sync::Arc;

#[derive(Debug, thiserror::Error)]
pub enum LoginError {
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("User account is inactive")]
    AccountInactive,

    #[error("Repository error: {0}")]
    RepositoryError(String),

    #[error("Token creation failed: {0}")]
    TokenCreationError(String),
}

pub struct LoginUseCase<R: AuthRepository> {
    auth_repo: Arc<R>,
    jwt_manager: Arc<JwtManager>,
}

impl<R: AuthRepository> LoginUseCase<R> {
    pub fn new(auth_repo: Arc<R>, jwt_manager: Arc<JwtManager>) -> Self {
        Self { auth_repo, jwt_manager }
    }

    pub async fn execute(
        &self,
        email: String,
        password: Option<String>,
        code: Option<String>,
    ) -> Result<AuthResponse, LoginError> {
        // Find user by email
        let mut user = self // Mut because we might consume code
            .auth_repo
            .find_by_email(&email)
            .await
            .map_err(|e| LoginError::RepositoryError(e.to_string()))?
            .ok_or(LoginError::InvalidCredentials)?;

        // Check if account is active
        // For code login, maybe allow inactive if it's the verification step?
        // No, verifying is separate. Login is for active users.
        if !user.is_active {
            return Err(LoginError::AccountInactive);
        }

        let mut credentials_valid = false;

        // Check Code
        if let Some(c) = code {
            if let Some(user_code) = &user.confirmation_code {
                if user_code == &c {
                    if let Some(expires_at) = user.confirmation_code_expires_at {
                        if chrono::Utc::now() <= expires_at {
                            credentials_valid = true;
                            // Consume code to prevent replay
                            user.confirmation_code = None;
                            user.confirmation_code_expires_at = None;
                            // We need to save this change!
                            self.auth_repo
                                .update_user(&user)
                                .await
                                .map_err(|e| LoginError::RepositoryError(e.to_string()))?;
                        }
                    }
                }
            }
        }
        // Check Password if code didn't validate (or wasn't provided)
        else if let Some(p) = password {
            if let Some(hash) = &user.password_hash {
                if PasswordManager::verify(&p, hash).unwrap_or(false) {
                    credentials_valid = true;
                }
            }
        }

        if !credentials_valid {
            return Err(LoginError::InvalidCredentials);
        }

        // Update last login
        self.auth_repo
            .update_last_login(*user.id.as_uuid())
            .await
            .map_err(|e| LoginError::RepositoryError(e.to_string()))?;

        // Generate tokens
        let access_token = self
            .jwt_manager
            .create_access_token(*user.id.as_uuid())
            .map_err(|e| LoginError::TokenCreationError(e.to_string()))?;

        let refresh_token = self
            .jwt_manager
            .create_refresh_token(*user.id.as_uuid())
            .map_err(|e| LoginError::TokenCreationError(e.to_string()))?;

        // Store refresh token
        let refresh_token_entity = RefreshToken::new(
            *user.id.as_uuid(),
            refresh_token.clone(),
            chrono::Utc::now() + self.jwt_manager.get_refresh_token_expiry(),
        );

        self.auth_repo
            .save_refresh_token(&refresh_token_entity)
            .await
            .map_err(|e| LoginError::RepositoryError(e.to_string()))?;

        Ok(AuthResponse {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: self.jwt_manager.get_access_token_expiry_seconds(),
            user: UserInfo {
                id: user.id.as_uuid().to_string(),
                email: user.email.as_str().to_string(),
                name: user.name.clone(),
            },
        })
    }
}
