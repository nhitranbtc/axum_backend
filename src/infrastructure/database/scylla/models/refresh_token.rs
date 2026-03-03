use chrono::{DateTime, Utc};
use uuid::Uuid;

/// ScyllaDB row for the `refresh_tokens` table.
#[derive(Debug, Clone)]
pub struct RefreshTokenRow {
    pub token_hash: String,
    pub user_id: Uuid,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}

impl RefreshTokenRow {
    pub fn new(
        token_hash: String,
        user_id: Uuid,
        expires_at: DateTime<Utc>,
        created_at: DateTime<Utc>,
        revoked_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self { token_hash, user_id, expires_at, created_at, revoked_at }
    }
}
