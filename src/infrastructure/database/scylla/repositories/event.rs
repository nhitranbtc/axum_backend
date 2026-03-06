use anyhow::{Context, Result};
use scylla::client::caching_session::CachingSession;
use std::sync::Arc;
use tracing::{debug, info};
use uuid::Uuid;

use crate::infrastructure::database::scylla::{
    connection::ScyllaSession, models::UserEventRow, operations::prelude::*,
};

/// Repository for managing user events in ScyllaDB (time-series data).
///
/// Holds only an `Arc<CachingSession>` — no `PreparedStatement` fields.
pub struct EventRepository {
    session: Arc<CachingSession>,
}

impl EventRepository {
    pub fn new(session: Arc<ScyllaSession>) -> Self {
        Self { session: session.session() }
    }

    /// Persist a user event.
    pub async fn save_event(&self, event: &UserEventRow) -> Result<()> {
        debug!("Saving event '{}' for user {}", event.event_type, event.user_id);

        event.insert().execute(&self.session).await.context("Failed to insert event")?;

        info!("Event '{}' saved for user {}", event.event_type, event.user_id);
        Ok(())
    }

    /// Returns the total number of events recorded for a user.
    pub async fn get_user_events_count(&self, user_id: Uuid) -> Result<i64> {
        debug!("Fetching event count for user {}", user_id);

        let result = execute_unpaged(&self.session, UserEventRow::COUNT_BY_USER_QUERY, (user_id,))
            .await
            .context("Failed to query user events")?
            .into_rows_result()
            .context("Failed to read event COUNT result")?;

        let (count,): (i64,) = result.first_row().context("No COUNT result row")?;

        Ok(count)
    }
}
