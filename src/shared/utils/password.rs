use argon2::{
    password_hash::{PasswordHash, PasswordHasher as _, PasswordVerifier, SaltString},
    Argon2,
};
use rand::rngs::OsRng;

#[derive(Debug, thiserror::Error)]
pub enum PasswordError {
    #[error("Failed to hash password: {0}")]
    HashError(String),

    #[error("Failed to verify password: {0}")]
    VerifyError(String),

    #[error("Invalid password")]
    InvalidPassword,
}

pub struct PasswordManager;

impl PasswordManager {
    /// Hash a password using Argon2
    pub fn hash(password: &str) -> Result<String, PasswordError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|e| PasswordError::HashError(e.to_string()))
    }

    /// Verify a password against a hash
    pub fn verify(password: &str, hash: &str) -> Result<bool, PasswordError> {
        let parsed_hash =
            PasswordHash::new(hash).map_err(|e| PasswordError::VerifyError(e.to_string()))?;

        // Use the parameters from the hash itself for verification (standard practice)
        let argon2 = Argon2::default();

        match argon2.verify_password(password.as_bytes(), &parsed_hash) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify_password() {
        let password = "my_secure_password";
        let hash = PasswordManager::hash(password).unwrap();

        assert!(PasswordManager::verify(password, &hash).unwrap());
        assert!(!PasswordManager::verify("wrong_password", &hash).unwrap());
    }
}
