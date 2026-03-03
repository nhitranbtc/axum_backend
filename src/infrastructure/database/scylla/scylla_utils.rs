use chrono::{DateTime, TimeZone, Utc};
use scylla::value::CqlTimestamp;

/// Convert a `CqlTimestamp` (milliseconds since epoch) to `DateTime<Utc>`.
pub(super) fn from_cql_ts(ts: CqlTimestamp) -> DateTime<Utc> {
    let ms = ts.0;
    let secs = ms / 1000;
    let nanos = ((ms % 1000) * 1_000_000) as u32;
    Utc.timestamp_opt(secs, nanos)
        .single()
        .unwrap_or_else(Utc::now)
}

/// Convert a `DateTime<Utc>` to `CqlTimestamp`.
pub(super) fn to_cql_ts(dt: DateTime<Utc>) -> CqlTimestamp {
    CqlTimestamp(dt.timestamp_millis())
}

/// Convert an optional `DateTime<Utc>` to an optional `CqlTimestamp`.
pub(super) fn opt_to_cql_ts(dt: Option<DateTime<Utc>>) -> Option<CqlTimestamp> {
    dt.map(to_cql_ts)
}

/// Convert an optional `CqlTimestamp` to an optional `DateTime<Utc>`.
pub(super) fn opt_from_cql_ts(ts: Option<CqlTimestamp>) -> Option<DateTime<Utc>> {
    ts.map(from_cql_ts)
}

/// Type alias for the full 12-column user row tuple returned by ScyllaDB.
pub(super) type UserRowTuple = (
    uuid::Uuid,          // user_id
    String,              // email
    String,              // name
    Option<String>,      // password_hash
    String,              // role
    bool,                // is_active
    bool,                // email_verified
    Option<String>,      // confirmation_code
    Option<CqlTimestamp>, // confirmation_code_expires_at
    Option<CqlTimestamp>, // last_login
    CqlTimestamp,        // created_at
    CqlTimestamp,        // updated_at
);
