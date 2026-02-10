use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::infrastructure::database::schema::refresh_tokens;

/// Database model for RefreshToken
///
/// Stores refresh tokens for JWT authentication with revocation support.
#[derive(Debug, Clone, Queryable, Selectable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = refresh_tokens)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RefreshTokenModel {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}

impl RefreshTokenModel {
    /// Create a new refresh token model
    pub fn new(id: Uuid, user_id: Uuid, token_hash: String, expires_at: DateTime<Utc>) -> Self {
        Self { id, user_id, token_hash, expires_at, created_at: Utc::now(), revoked_at: None }
    }

    /// Check if the token is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Check if the token is revoked
    pub fn is_revoked(&self) -> bool {
        self.revoked_at.is_some()
    }

    /// Check if the token is valid (not expired and not revoked)
    pub fn is_valid(&self) -> bool {
        !self.is_expired() && !self.is_revoked()
    }

    /// Revoke the token
    pub fn revoke(&mut self) {
        self.revoked_at = Some(Utc::now());
    }
}

// Backward compatibility alias (will be deprecated)
#[deprecated(since = "0.2.0", note = "Use `RefreshTokenModel` instead")]
pub type DbRefreshToken = RefreshTokenModel;
