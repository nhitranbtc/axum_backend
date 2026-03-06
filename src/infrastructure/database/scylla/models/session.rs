use chrono::{DateTime, Utc};
use scylla::value::CqlTimestamp;
use scylla_macros::{DeserializeRow, SerializeRow};
use uuid::Uuid;

// ── ScyllaDB row for the `user_sessions` table ────────────────────────────────

#[derive(Debug, Clone, SerializeRow, DeserializeRow)]
pub struct UserSessionRow {
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub data: String, // JSON-serialised session data
    pub expires_at: CqlTimestamp,
    pub created_at: CqlTimestamp,
}

impl UserSessionRow {
    // ── Static query constants ────────────────────────────────────────────────
    pub const INSERT_QUERY: &'static str =
        "INSERT INTO user_sessions (session_id, user_id, data, expires_at, created_at) \
         VALUES (?, ?, ?, ?, ?)";

    pub const DELETE_QUERY: &'static str = "DELETE FROM user_sessions WHERE session_id = ?";

    // ── Factory ───────────────────────────────────────────────────────────────

    /// Build a new session row with a generated `session_id` and computed
    /// `expires_at` timestamp.
    pub fn new(user_id: Uuid, data: String, ttl_seconds: i64) -> Self {
        let now = Utc::now();
        let expires = now + chrono::Duration::seconds(ttl_seconds);
        Self {
            session_id: Uuid::new_v4(),
            user_id,
            data,
            expires_at: Self::ts(expires),
            created_at: Self::ts(now),
        }
    }

    /// Returns `true` if the session has passed its expiry time.
    pub fn is_expired(&self) -> bool {
        Utc::now().timestamp_millis() > self.expires_at.0
    }

    // ── Timestamp helpers ─────────────────────────────────────────────────────

    pub fn ts(dt: DateTime<Utc>) -> CqlTimestamp {
        CqlTimestamp(dt.timestamp_millis())
    }
}

// ── BaseModel + Model traits ───────────────────────────────────────────────────

use crate::infrastructure::database::scylla::operations::model::{BaseModel, Model};

impl BaseModel for UserSessionRow {
    type PrimaryKey = (Uuid,);
    type PartitionKey = (Uuid,);

    const TABLE_NAME: &'static str = "user_sessions";
    const FIND_ALL_QUERY: &'static str =
        "SELECT session_id, user_id, data, expires_at, created_at FROM user_sessions";
    const FIND_BY_PRIMARY_KEY_QUERY: &'static str =
        "SELECT session_id, user_id, data, expires_at, created_at \
         FROM user_sessions WHERE session_id = ?";
    const FIND_BY_PARTITION_KEY_QUERY: &'static str =
        "SELECT session_id, user_id, data, expires_at, created_at \
         FROM user_sessions WHERE session_id = ?";

    fn primary_key_values(&self) -> (Uuid,) {
        (self.session_id,)
    }
    fn partition_key_values(&self) -> (Uuid,) {
        (self.session_id,)
    }
}

impl Model for UserSessionRow {
    const INSERT_QUERY: &'static str = UserSessionRow::INSERT_QUERY;
    const INSERT_IF_NOT_EXISTS_QUERY: &'static str =
        "INSERT INTO user_sessions (session_id, user_id, data, expires_at, created_at) \
         VALUES (?, ?, ?, ?, ?) IF NOT EXISTS";
    const UPDATE_QUERY: &'static str =
        "UPDATE user_sessions SET data = ?, expires_at = ? WHERE session_id = ?";
    const DELETE_QUERY: &'static str = UserSessionRow::DELETE_QUERY;
    const DELETE_BY_PARTITION_KEY_QUERY: &'static str = UserSessionRow::DELETE_QUERY;
}
