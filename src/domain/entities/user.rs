use crate::domain::{
    errors::DomainError,
    value_objects::{Email, UserId, UserRole},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// User domain entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub email: Email,
    pub name: String,
    pub password_hash: Option<String>, // Now optional
    pub role: UserRole,
    pub is_active: bool,
    pub is_email_verified: bool,
    pub confirmation_code: Option<String>,
    pub confirmation_code_expires_at: Option<DateTime<Utc>>,
    pub last_login: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    /// Create a new user (inactive, no password, with confirmation code)
    pub fn new(email: Email, name: String) -> Result<Self, DomainError> {
        // Validate name
        if name.trim().is_empty() {
            return Err(DomainError::InvalidName);
        }

        if name.len() > 255 {
            return Err(DomainError::InvalidUserData(
                "Name must be less than 255 characters".to_string(),
            ));
        }

        let now = Utc::now();

        // 6-digit code generation logic should ideally be in a service, using basic random here or placeholder
        // But Domain Entity shouldn't depend on external RNG service easily.
        // We'll set it to None here and let the Use Case set it?
        // Or we pass it in. User::new(email, name, code, code_expiry)
        // Let's modify User::new signature to fit the "Register" use case.

        Ok(Self {
            id: UserId::new(),
            email,
            name: name.trim().to_string(),
            password_hash: None,
            role: UserRole::default(),
            is_active: false,
            is_email_verified: false,
            confirmation_code: None, // Set by `set_confirmation_code`
            confirmation_code_expires_at: None,
            last_login: None,
            created_at: now,
            updated_at: now,
        })
    }

    /// Set confirmation code
    pub fn set_confirmation_code(&mut self, code: String, expires_at: DateTime<Utc>) {
        self.confirmation_code = Some(code);
        self.confirmation_code_expires_at = Some(expires_at);
        self.updated_at = Utc::now();
    }

    /// Verify email
    pub fn verify_email(&mut self) {
        self.is_email_verified = true;
        self.is_active = true; // Activate user upon email verification? Yes for now.
                               // Keep confirmation code for Set Password step or subsequent login?
                               // We generally shouldn't reuse codes, but for this specific flow "Register -> Verify -> Set Password",
                               // the code acts as the temporary credential.
        self.updated_at = Utc::now();
    }

    /// Set password
    pub fn set_password(&mut self, hash: String) {
        self.password_hash = Some(hash);
        self.updated_at = Utc::now();
    }

    /// Create user with existing ID (for loading from database)
    #[allow(clippy::too_many_arguments)]
    pub fn from_existing(
        id: UserId,
        email: Email,
        name: String,
        password_hash: Option<String>,
        role: UserRole,
        is_active: bool,
        is_email_verified: bool,
        confirmation_code: Option<String>,
        confirmation_code_expires_at: Option<DateTime<Utc>>,
        last_login: Option<DateTime<Utc>>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            email,
            name,
            password_hash,
            role,
            is_active,
            is_email_verified,
            confirmation_code,
            confirmation_code_expires_at,
            last_login,
            created_at,
            updated_at,
        }
    }

    /// Update user name
    pub fn update_name(&mut self, new_name: String) -> Result<(), DomainError> {
        if new_name.trim().is_empty() {
            return Err(DomainError::InvalidName);
        }

        if new_name.len() > 255 {
            return Err(DomainError::InvalidUserData(
                "Name must be less than 255 characters".to_string(),
            ));
        }

        self.name = new_name.trim().to_string();
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Update user email
    pub fn update_email(&mut self, new_email: Email) {
        self.email = new_email;
        self.is_email_verified = false; // Reset verification on email change
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_user() {
        let email = Email::parse("test@example.com").unwrap();
        let user = User::new(email, "John Doe".to_string()).unwrap();

        assert_eq!(user.name, "John Doe");
        assert_eq!(user.email.as_str(), "test@example.com");
    }

    #[test]
    fn test_create_user_empty_name() {
        let email = Email::parse("test@example.com").unwrap();
        let result = User::new(email, "   ".to_string());

        assert!(result.is_err());
    }

    #[test]
    fn test_update_name() {
        let email = Email::parse("test@example.com").unwrap();
        let mut user =
            User::new(email, "John Doe".to_string()).unwrap();

        user.update_name("Jane Doe".to_string()).unwrap();
        assert_eq!(user.name, "Jane Doe");
    }

    #[test]
    fn test_update_email() {
        let email = Email::parse("test@example.com").unwrap();
        let mut user =
            User::new(email, "John Doe".to_string()).unwrap();

        let new_email = Email::parse("new@example.com").unwrap();
        user.update_email(new_email);
        assert_eq!(user.email.as_str(), "new@example.com");
    }
}
