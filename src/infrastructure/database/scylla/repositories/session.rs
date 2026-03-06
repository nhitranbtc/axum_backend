use anyhow::{Context, Result};
use scylla::client::caching_session::CachingSession;
use std::sync::Arc;
use tracing::{debug, info};
use uuid::Uuid;

use crate::infrastructure::database::scylla::{
    connection::ScyllaSession,
    models::UserSessionRow,
    operations::prelude::*,
};

/// Repository for managing user sessions in ScyllaDB.
///
/// Holds only an `Arc<CachingSession>` — no `PreparedStatement` fields.
pub struct SessionRepository {
    session: Arc<CachingSession>,
}

impl SessionRepository {
    pub fn new(session: Arc<ScyllaSession>) -> Self {
        Self { session: session.session() }
    }

    /// Save a user session.
    pub async fn save_session(&self, row: &UserSessionRow) -> Result<()> {
        debug!("Saving session {} for user {}", row.session_id, row.user_id);

        row.insert()
            .execute(&self.session)
            .await
            .context("Failed to insert session")?;

        info!("Session saved: {} for user {}", row.session_id, row.user_id);
        Ok(())
    }

    /// Delete a session by its primary key.
    pub async fn delete_session(&self, session_id: Uuid) -> Result<()> {
        debug!("Deleting session {}", session_id);

        UserSessionRow::delete_by_query(UserSessionRow::DELETE_QUERY, (session_id,))
            .execute(&self.session)
            .await
            .context("Failed to delete session")?;

        info!("Session deleted: {}", session_id);
        Ok(())
    }
}
