/// Domain-specific errors (business rule violations)
#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("Invalid email format: {0}")]
    InvalidEmail(String),

    #[error("Invalid name: name cannot be empty")]
    InvalidName,

    #[error("Invalid password: {0}")]
    InvalidPassword(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Invalid user data: {0}")]
    InvalidUserData(String),
}
