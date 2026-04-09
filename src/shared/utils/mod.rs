pub mod jwt;
pub mod password;

use chrono::{DateTime, Utc};
use rand::rngs::OsRng;
use rand::RngCore;

/// Get current UTC timestamp
pub fn now() -> DateTime<Utc> {
    Utc::now()
}

/// Validate email format (basic validation)
pub fn is_valid_email(email: &str) -> bool {
    email.contains('@') && email.len() > 3 && !email.starts_with('@')
}

/// Generate a cryptographically secure confirmation code.
///
/// Uses `OsRng` (CSPRNG) and produces a 32-byte hex token (64 characters,
/// 256 bits of entropy).
pub fn generate_confirmation_code() -> String {
    let mut buf = [0u8; 32];
    OsRng.fill_bytes(&mut buf);
    hex::encode(buf)
}

/// Hash a token string with SHA-256 and return the hex digest.
///
/// Used for hashing refresh tokens before database storage so that a DB
/// breach does not expose live tokens.
pub fn hash_token(token: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_email() {
        assert!(is_valid_email("test@example.com"));
        assert!(!is_valid_email("invalid"));
        assert!(!is_valid_email("@example.com"));
    }

    #[test]
    fn confirmation_code_is_64_hex_chars() {
        let code = generate_confirmation_code();
        assert_eq!(code.len(), 64);
        assert!(code.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn confirmation_codes_are_unique() {
        let a = generate_confirmation_code();
        let b = generate_confirmation_code();
        assert_ne!(a, b);
    }

    #[test]
    fn hash_token_produces_consistent_hex() {
        let hash = hash_token("test-token");
        assert_eq!(hash.len(), 64); // SHA-256 hex = 64 chars
        assert_eq!(hash, hash_token("test-token"));
        assert_ne!(hash, hash_token("other-token"));
    }
}
