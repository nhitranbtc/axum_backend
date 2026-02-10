use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,        // Subject (user ID)
    pub exp: i64,           // Expiration time
    pub iat: i64,           // Issued at
    pub jti: String,        // JWT ID (unique identifier)
    pub token_type: String, // "access" or "refresh"
    pub iss: String,        // Issuer
    pub aud: String,        // Audience
}

#[derive(Debug, thiserror::Error)]
pub enum JwtError {
    #[error("Failed to create token: {0}")]
    TokenCreation(String),

    #[error("Invalid token: {0}")]
    InvalidToken(String),

    #[error("Token expired")]
    TokenExpired,
}

pub struct JwtManager {
    secret: String,
    access_token_expiry: Duration,
    refresh_token_expiry: Duration,
    issuer: String,
    audience: String,
}

impl JwtManager {
    pub fn new(
        secret: String,
        access_token_expiry: i64,
        refresh_token_expiry: i64,
        issuer: String,
        audience: String,
    ) -> Self {
        // Validate secret strength (minimum 32 bytes for HS256)
        if secret.len() < 32 {
            panic!("JWT secret must be at least 32 characters for security");
        }

        Self {
            secret,
            access_token_expiry: Duration::seconds(access_token_expiry),
            refresh_token_expiry: Duration::seconds(refresh_token_expiry),
            issuer,
            audience,
        }
    }

    pub fn create_access_token(&self, user_id: Uuid) -> Result<String, JwtError> {
        let now = Utc::now();
        let expiry = now + self.access_token_expiry;

        let claims = Claims {
            sub: user_id.to_string(),
            exp: expiry.timestamp(),
            iat: now.timestamp(),
            jti: Uuid::new_v4().to_string(),
            token_type: "access".to_string(),
            iss: self.issuer.clone(),
            aud: self.audience.clone(),
        };

        let mut header = Header::new(Algorithm::HS256);
        header.typ = Some("JWT".to_string());

        encode(&header, &claims, &EncodingKey::from_secret(self.secret.as_bytes()))
            .map_err(|e| JwtError::TokenCreation(e.to_string()))
    }

    pub fn create_refresh_token(&self, user_id: Uuid) -> Result<String, JwtError> {
        let now = Utc::now();
        let expiry = now + self.refresh_token_expiry;

        let claims = Claims {
            sub: user_id.to_string(),
            exp: expiry.timestamp(),
            iat: now.timestamp(),
            jti: Uuid::new_v4().to_string(),
            token_type: "refresh".to_string(),
            iss: self.issuer.clone(),
            aud: self.audience.clone(),
        };

        let mut header = Header::new(Algorithm::HS256);
        header.typ = Some("JWT".to_string());

        encode(&header, &claims, &EncodingKey::from_secret(self.secret.as_bytes()))
            .map_err(|e| JwtError::TokenCreation(e.to_string()))
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims, JwtError> {
        let mut validation = Validation::new(Algorithm::HS256);

        // Strict validation settings
        validation.set_required_spec_claims(&["exp", "sub", "iat", "jti", "iss", "aud"]);
        validation.validate_exp = true;
        validation.set_issuer(&[&self.issuer]);
        validation.set_audience(&[&self.audience]);

        decode::<Claims>(token, &DecodingKey::from_secret(self.secret.as_bytes()), &validation)
            .map(|data| data.claims)
            .map_err(|e| {
                if e.to_string().contains("ExpiredSignature") {
                    JwtError::TokenExpired
                } else {
                    JwtError::InvalidToken(e.to_string())
                }
            })
    }

    pub fn get_access_token_expiry_seconds(&self) -> i64 {
        self.access_token_expiry.num_seconds()
    }

    pub fn get_refresh_token_expiry(&self) -> Duration {
        self.refresh_token_expiry
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_verify_access_token() {
        let jwt_manager = JwtManager::new(
            "test_secret_that_is_long_enough_32chars".to_string(),
            3600,
            86400,
            "test-issuer".to_string(),
            "test-audience".to_string(),
        );
        let user_id = Uuid::new_v4();

        let token = jwt_manager.create_access_token(user_id).unwrap();
        let claims = jwt_manager.verify_token(&token).unwrap();

        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.token_type, "access");
        assert_eq!(claims.iss, "test-issuer");
        assert_eq!(claims.aud, "test-audience");
    }

    #[test]
    fn test_create_and_verify_refresh_token() {
        let jwt_manager = JwtManager::new(
            "test_secret_that_is_long_enough_32chars".to_string(),
            3600,
            86400,
            "test-issuer".to_string(),
            "test-audience".to_string(),
        );
        let user_id = Uuid::new_v4();

        let token = jwt_manager.create_refresh_token(user_id).unwrap();
        let claims = jwt_manager.verify_token(&token).unwrap();

        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.token_type, "refresh");
        assert_eq!(claims.iss, "test-issuer");
        assert_eq!(claims.aud, "test-audience");
    }
}
