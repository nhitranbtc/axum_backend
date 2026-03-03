use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// User session stored in ScyllaDB
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSessionRow {
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub data: String, // JSON-serialized session data
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl UserSessionRow {
    pub fn new(user_id: Uuid, data: String, ttl_seconds: i64) -> Self {
        let now = Utc::now();
        Self {
            session_id: Uuid::new_v4(),
            user_id,
            data,
            expires_at: now + chrono::Duration::seconds(ttl_seconds),
            created_at: now,
        }
    }
    
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}
