pub mod jwt;
pub mod password;

use chrono::{DateTime, Utc};
use rand::rngs::OsRng;
use rand::Rng;

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
/// Uses `OsRng` (CSPRNG) and produces an 8-character uppercase alphanumeric
/// string (36^8 ≈ 2.8 trillion combinations).
pub fn generate_confirmation_code() -> String {
    (0..8)
        .map(|_| {
            let idx: u8 = OsRng.gen_range(0..36);
            if idx < 10 {
                (b'0' + idx) as char
            } else {
                (b'A' + idx - 10) as char
            }
        })
        .collect()
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
    fn confirmation_code_is_8_alphanumeric_chars() {
        let code = generate_confirmation_code();
        assert_eq!(code.len(), 8);
        assert!(code.chars().all(|c| c.is_ascii_alphanumeric()));
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
