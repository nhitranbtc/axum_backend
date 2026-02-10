use crate::shared::errors::AppError;
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct Recipient {
    pub email: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub enum EmailType {
    Welcome(String),       // Name
    Confirmation(String),  // Code
    PasswordReset(String), // Code (was Token, but now Code for forgot pass flow)
}

impl EmailType {
    pub fn subject(&self) -> String {
        match self {
            EmailType::Welcome(_) => "Welcome to Axum Backend!".to_string(),
            EmailType::Confirmation(_) => "Confirm your registration".to_string(),
            EmailType::PasswordReset(_) => "Reset your password".to_string(),
        }
    }

    pub fn body(&self) -> String {
        match self {
            EmailType::Welcome(name) => format!("Hello {}, welcome to our platform!", name),
            EmailType::Confirmation(code) => format!("Your confirmation code is: {}", code),
            EmailType::PasswordReset(code) => format!("Your password reset code is: {}", code),
        }
    }
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait EmailService: Send + Sync {
    async fn send(&self, recipient: Recipient, email_type: EmailType) -> Result<(), AppError>;
}
