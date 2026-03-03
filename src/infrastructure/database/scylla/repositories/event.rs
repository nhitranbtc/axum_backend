use anyhow::{Context, Result};
use scylla::statement::prepared::PreparedStatement;
use std::sync::Arc;
use tracing::{debug, info};
use uuid::Uuid;

use super::super::connection::ScyllaSession;
use super::super::models::UserEventRow;

/// Repository for managing user events in ScyllaDB.
///
/// Queries are prepared on construction so ScyllaDB only parses and plans them once.
pub struct EventRepository {
    session: Arc<ScyllaSession>,
    ps_insert: PreparedStatement,
    ps_count: PreparedStatement,
}

impl EventRepository {
    /// Create a new event repository, preparing all statements upfront.
    pub async fn new(session: Arc<ScyllaSession>) -> Result<Self> {
        let ps_insert = session
            .session()
            .prepare(
                "INSERT INTO user_events (user_id, event_id, event_type, event_data, created_at) \
                 VALUES (?, ?, ?, ?, ?)",
            )
            .await
            .context("Failed to prepare event INSERT")?;

        let ps_count = session
            .session()
            .prepare("SELECT COUNT(*) FROM user_events WHERE user_id = ?")
            .await
            .context("Failed to prepare event COUNT")?;

        Ok(Self { session, ps_insert, ps_count })
    }

    /// Save a user event.
    pub async fn save_event(&self, event: &UserEventRow) -> Result<()> {
        debug!("Saving event for user {}: {}", event.user_id, event.event_type);

        let created_at_ms = event.created_at.timestamp_millis();

        self.session
            .session()
            .execute_unpaged(
                &self.ps_insert,
                (
                    event.user_id,
                    event.event_id,
                    &event.event_type,
                    &event.event_data,
                    scylla::value::CqlTimestamp(created_at_ms),
                ),
            )
            .await
            .context("Failed to insert event")?;

        info!("Event saved: {} for user {}", event.event_type, event.user_id);
        Ok(())
    }

    /// Return the total number of events recorded for a user.
    pub async fn get_user_events_count(&self, user_id: Uuid) -> Result<i64> {
        debug!("Fetching event count for user {}", user_id);

        let result = self
            .session
            .session()
            .execute_unpaged(&self.ps_count, (user_id,))
            .await
            .context("Failed to query user events")?;

        let rows = result
            .into_rows_result()
            .context("Failed to read event COUNT result")?;

        let count: i64 = rows
            .rows::<(i64,)>()
            .context("Failed to deserialize COUNT row")?
            .next()
            .ok_or_else(|| anyhow::anyhow!("No COUNT result row"))??
            .0;

        Ok(count)
    }
}
