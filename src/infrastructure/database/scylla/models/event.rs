use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// User event stored in ScyllaDB (time-series data).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserEventRow {
    pub user_id: Uuid,
    pub event_id: Uuid,   // UUID v4
    pub event_type: String,
    pub event_data: String, // JSON-serialised event payload
    pub created_at: DateTime<Utc>,
}

impl UserEventRow {
    pub fn new(user_id: Uuid, event_type: String, event_data: String) -> Self {
        let event_id = Uuid::new_v4();
        let created_at = Utc::now();

        Self { user_id, event_id, event_type, event_data, created_at }
    }
}
