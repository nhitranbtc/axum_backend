use chrono::{DateTime, Utc};
use scylla::value::CqlTimestamp;
use scylla_macros::{DeserializeRow, SerializeRow};
use uuid::Uuid;

use crate::infrastructure::database::scylla::operations::model::{BaseModel, Model};

// ── ScyllaDB row for the `users` table ───────────────────────────────────────
//
// Fields use `CqlTimestamp` (ms-since-epoch i64) for lossless wire transport.
// Use `UserRow::from_cql` / `as_cql` helpers to convert to/from `DateTime<Utc>`.
//
// All static CQL query strings live here as associated constants so that
// repositories never hard-code SQL themselves (charybdis-style architecture).

#[derive(Debug, Clone, SerializeRow, DeserializeRow)]
pub struct UserRow {
    pub user_id: Uuid,
    pub email: String,
    pub name: String,
    pub password_hash: Option<String>,
    pub role: String,
    pub is_active: bool,
    pub email_verified: bool,
    pub confirmation_code: Option<String>,
    pub confirmation_code_expires_at: Option<CqlTimestamp>,
    pub last_login: Option<CqlTimestamp>,
    pub created_at: CqlTimestamp,
    pub updated_at: CqlTimestamp,
}

impl UserRow {
    // ── Select column list ────────────────────────────────────────────────────
    /// Canonical column list used in every SELECT (single source of truth).
    pub const COLUMNS: &'static str =
        "user_id, email, name, password_hash, role, is_active, \
         email_verified, confirmation_code, confirmation_code_expires_at, \
         last_login, created_at, updated_at";

    // ── Static query constants (charybdis-style) ──────────────────────────────
    pub const INSERT_QUERY: &'static str =
        "INSERT INTO users (user_id, email, name, password_hash, role, is_active, \
         email_verified, confirmation_code, confirmation_code_expires_at, \
         last_login, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";

    pub const FIND_BY_PRIMARY_KEY_QUERY: &'static str =
        "SELECT user_id, email, name, password_hash, role, is_active, \
         email_verified, confirmation_code, confirmation_code_expires_at, \
         last_login, created_at, updated_at FROM users WHERE user_id = ?";

    pub const FIND_BY_EMAIL_QUERY: &'static str =
        "SELECT user_id, email, name, password_hash, role, is_active, \
         email_verified, confirmation_code, confirmation_code_expires_at, \
         last_login, created_at, updated_at FROM users WHERE email = ? ALLOW FILTERING";

    pub const FIND_ALL_QUERY: &'static str =
        "SELECT user_id, email, name, password_hash, role, is_active, \
         email_verified, confirmation_code, confirmation_code_expires_at, \
         last_login, created_at, updated_at FROM users LIMIT ?";

    pub const UPDATE_QUERY: &'static str =
        "UPDATE users SET name = ?, email = ?, password_hash = ?, role = ?, \
         is_active = ?, email_verified = ?, confirmation_code = ?, \
         confirmation_code_expires_at = ?, updated_at = ? WHERE user_id = ?";

    pub const UPDATE_LAST_LOGIN_QUERY: &'static str =
        "UPDATE users SET last_login = ?, updated_at = ? WHERE user_id = ?";

    pub const DELETE_QUERY: &'static str = "DELETE FROM users WHERE user_id = ?";

    pub const COUNT_QUERY: &'static str = "SELECT COUNT(*) FROM users";

    pub const DELETE_ALL_QUERY: &'static str = "TRUNCATE users";

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

impl BaseModel for UserRow {
    type PrimaryKey = (Uuid,);
    type PartitionKey = (Uuid,);

    const TABLE_NAME: &'static str = "users";
    const FIND_ALL_QUERY: &'static str = UserRow::FIND_ALL_QUERY;
    const FIND_BY_PRIMARY_KEY_QUERY: &'static str = UserRow::FIND_BY_PRIMARY_KEY_QUERY;
    const FIND_BY_PARTITION_KEY_QUERY: &'static str = UserRow::FIND_BY_PRIMARY_KEY_QUERY;

    fn primary_key_values(&self) -> (Uuid,) { (self.user_id,) }
    fn partition_key_values(&self) -> (Uuid,) { (self.user_id,) }
}

impl Model for UserRow {
    const INSERT_QUERY: &'static str = UserRow::INSERT_QUERY;
    const INSERT_IF_NOT_EXISTS_QUERY: &'static str =
        "INSERT INTO users (user_id, email, name, password_hash, role, is_active, \
         email_verified, confirmation_code, confirmation_code_expires_at, \
         last_login, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) IF NOT EXISTS";
    const UPDATE_QUERY: &'static str = UserRow::UPDATE_QUERY;
    const DELETE_QUERY: &'static str = UserRow::DELETE_QUERY;
    const DELETE_BY_PARTITION_KEY_QUERY: &'static str = UserRow::DELETE_QUERY;
}

