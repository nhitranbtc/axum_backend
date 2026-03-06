use chrono::{DateTime, Utc};
use scylla::value::CqlTimestamp;
use scylla_macros::{DeserializeRow, SerializeRow};
use uuid::Uuid;

// ── ScyllaDB row for the `refresh_tokens` table ───────────────────────────────

#[derive(Debug, Clone, SerializeRow, DeserializeRow)]
pub struct RefreshTokenRow {
    pub token_hash: String,
    pub user_id: Uuid,
    pub expires_at: CqlTimestamp,
    pub created_at: CqlTimestamp,
    pub revoked_at: Option<CqlTimestamp>,
}

impl RefreshTokenRow {
    // ── Static query constants ────────────────────────────────────────────────
    pub const INSERT_QUERY: &'static str =
        "INSERT INTO refresh_tokens (token_hash, user_id, expires_at, created_at, revoked_at) \
         VALUES (?, ?, ?, ?, ?)";

    pub const FIND_BY_PRIMARY_KEY_QUERY: &'static str =
        "SELECT token_hash, user_id, expires_at, created_at, revoked_at \
         FROM refresh_tokens WHERE token_hash = ?";

    pub const FIND_HASHES_BY_USER_QUERY: &'static str =
        "SELECT token_hash FROM refresh_tokens WHERE user_id = ? ALLOW FILTERING";

    pub const REVOKE_QUERY: &'static str =
        "UPDATE refresh_tokens SET revoked_at = ? WHERE token_hash = ?";

    // ── Timestamp helpers ─────────────────────────────────────────────────────

    pub fn ts(dt: DateTime<Utc>) -> CqlTimestamp {
        CqlTimestamp(dt.timestamp_millis())
    }

    pub fn opt_ts(dt: Option<DateTime<Utc>>) -> Option<CqlTimestamp> {
        dt.map(Self::ts)
    }

    pub fn from_ts(ts: CqlTimestamp) -> DateTime<Utc> {
        let secs = ts.0 / 1_000;
        let nanos = ((ts.0 % 1_000) * 1_000_000) as u32;
        DateTime::from_timestamp(secs, nanos).unwrap_or_else(Utc::now)
    }

    pub fn from_opt_ts(ts: Option<CqlTimestamp>) -> Option<DateTime<Utc>> {
        ts.map(Self::from_ts)
    }
}

// ── BaseModel + Model traits ───────────────────────────────────────────────────

use crate::infrastructure::database::scylla::operations::model::{BaseModel, Model};

impl BaseModel for RefreshTokenRow {
    type PrimaryKey = (String,);
    type PartitionKey = (String,);

    const TABLE_NAME: &'static str = "refresh_tokens";
    const FIND_ALL_QUERY: &'static str =
        "SELECT token_hash, user_id, expires_at, created_at, revoked_at FROM refresh_tokens";
    const FIND_BY_PRIMARY_KEY_QUERY: &'static str = RefreshTokenRow::FIND_BY_PRIMARY_KEY_QUERY;
    const FIND_BY_PARTITION_KEY_QUERY: &'static str = RefreshTokenRow::FIND_BY_PRIMARY_KEY_QUERY;

    fn primary_key_values(&self) -> (String,) { (self.token_hash.clone(),) }
    fn partition_key_values(&self) -> (String,) { (self.token_hash.clone(),) }
}

impl Model for RefreshTokenRow {
    const INSERT_QUERY: &'static str = RefreshTokenRow::INSERT_QUERY;
    const INSERT_IF_NOT_EXISTS_QUERY: &'static str =
        "INSERT INTO refresh_tokens (token_hash, user_id, expires_at, created_at, revoked_at) \
         VALUES (?, ?, ?, ?, ?) IF NOT EXISTS";
    const UPDATE_QUERY: &'static str = RefreshTokenRow::REVOKE_QUERY;
    const DELETE_QUERY: &'static str =
        "DELETE FROM refresh_tokens WHERE token_hash = ?";
    const DELETE_BY_PARTITION_KEY_QUERY: &'static str =
        "DELETE FROM refresh_tokens WHERE token_hash = ?";
}

