use scylla::value::CqlTimestamp;
use scylla_macros::{DeserializeRow, SerializeRow};
use uuid::Uuid;

// ── ScyllaDB row for the `user_events` table (time-series) ───────────────────

#[derive(Debug, Clone, SerializeRow, DeserializeRow)]
pub struct UserEventRow {
    pub user_id: Uuid,
    pub event_id: Uuid,        // UUID v4 — doubles as a unique row key
    pub event_type: String,
    pub event_data: String,    // JSON-serialised event payload
    pub created_at: CqlTimestamp,
}

impl UserEventRow {
    // ── Static query constants ────────────────────────────────────────────────
    pub const INSERT_QUERY: &'static str =
        "INSERT INTO user_events (user_id, event_id, event_type, event_data, created_at) \
         VALUES (?, ?, ?, ?, ?)";

    pub const COUNT_BY_USER_QUERY: &'static str =
        "SELECT COUNT(*) FROM user_events WHERE user_id = ?";

    // ── Factory ───────────────────────────────────────────────────────────────

    /// Build a new event row with a generated `event_id` and the current time.
    pub fn new(user_id: Uuid, event_type: String, event_data: String) -> Self {
        Self {
            user_id,
            event_id: Uuid::new_v4(),
            event_type,
            event_data,
            created_at: CqlTimestamp(chrono::Utc::now().timestamp_millis()),
        }
    }
}

// ── BaseModel + Model traits ───────────────────────────────────────────────────

use crate::infrastructure::database::scylla::operations::model::{BaseModel, Model};

impl BaseModel for UserEventRow {
    type PrimaryKey = (Uuid, Uuid);
    type PartitionKey = (Uuid,);

    const TABLE_NAME: &'static str = "user_events";
    const FIND_ALL_QUERY: &'static str =
        "SELECT user_id, event_id, event_type, event_data, created_at FROM user_events";
    const FIND_BY_PRIMARY_KEY_QUERY: &'static str =
        "SELECT user_id, event_id, event_type, event_data, created_at \
         FROM user_events WHERE user_id = ? AND event_id = ?";
    const FIND_BY_PARTITION_KEY_QUERY: &'static str =
        "SELECT user_id, event_id, event_type, event_data, created_at \
         FROM user_events WHERE user_id = ?";

    fn primary_key_values(&self) -> (Uuid, Uuid) { (self.user_id, self.event_id) }
    fn partition_key_values(&self) -> (Uuid,) { (self.user_id,) }
}

impl Model for UserEventRow {
    const INSERT_QUERY: &'static str = UserEventRow::INSERT_QUERY;
    const INSERT_IF_NOT_EXISTS_QUERY: &'static str =
        "INSERT INTO user_events (user_id, event_id, event_type, event_data, created_at) \
         VALUES (?, ?, ?, ?, ?) IF NOT EXISTS";
    const UPDATE_QUERY: &'static str =
        "UPDATE user_events SET event_type = ?, event_data = ? \
         WHERE user_id = ? AND event_id = ?";
    const DELETE_QUERY: &'static str =
        "DELETE FROM user_events WHERE user_id = ? AND event_id = ?";
    const DELETE_BY_PARTITION_KEY_QUERY: &'static str =
        "DELETE FROM user_events WHERE user_id = ?";
}

