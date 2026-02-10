use crate::domain::errors::DomainError;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Email value object - ensures email validity
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Email(String);

impl Email {
    /// Parse and validate an email address
    pub fn parse(email: impl Into<String>) -> Result<Self, DomainError> {
        let email = email.into();

        // Basic email validation
        if !email.contains('@') || email.len() < 3 {
            return Err(DomainError::InvalidEmail(email));
        }

        // Additional validation rules
        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
            return Err(DomainError::InvalidEmail(email));
        }

        Ok(Self(email.to_lowercase()))
    }

    /// Get the email as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Convert to owned String
    pub fn into_string(self) -> String {
        self.0
    }
}

impl fmt::Display for Email {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_email() {
        let email = Email::parse("test@example.com").unwrap();
        assert_eq!(email.as_str(), "test@example.com");
    }

    #[test]
    fn test_email_lowercase() {
        let email = Email::parse("Test@Example.COM").unwrap();
        assert_eq!(email.as_str(), "test@example.com");
    }

    #[test]
    fn test_invalid_email_no_at() {
        assert!(Email::parse("invalid").is_err());
    }

    #[test]
    fn test_invalid_email_empty_local() {
        assert!(Email::parse("@example.com").is_err());
    }
}
