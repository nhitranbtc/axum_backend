use anyhow::{Context, Result};
use scylla::statement::prepared::PreparedStatement;
use std::sync::Arc;
use tracing::{debug, info};
use uuid::Uuid;

use super::super::connection::ScyllaSession;
use super::super::models::UserSessionRow;
use super::super::scylla_utils::{opt_to_cql_ts, to_cql_ts};

/// Repository for managing user sessions in ScyllaDB.
///
/// Queries are prepared on construction so ScyllaDB only parses and plans them once.
pub struct SessionRepository {
    session: Arc<ScyllaSession>,
    ps_insert: PreparedStatement,
    ps_delete: PreparedStatement,
}

impl SessionRepository {
    /// Create a new session repository, preparing all statements upfront.
    pub async fn new(session: Arc<ScyllaSession>) -> Result<Self> {
        let ps_insert = session
            .session()
            .prepare(
                "INSERT INTO user_sessions (session_id, user_id, data, expires_at, created_at) \
                 VALUES (?, ?, ?, ?, ?)",
            )
            .await
            .context("Failed to prepare session INSERT")?;

        let ps_delete = session
            .session()
            .prepare("DELETE FROM user_sessions WHERE session_id = ?")
            .await
            .context("Failed to prepare session DELETE")?;

        Ok(Self { session, ps_insert, ps_delete })
    }

    /// Save a user session.
    pub async fn save_session(&self, session_data: &UserSessionRow) -> Result<()> {
        debug!(
            "Saving session {} for user {}",
            session_data.session_id, session_data.user_id
        );

        self.session
            .session()
            .execute_unpaged(
                &self.ps_insert,
                (
                    session_data.session_id,
                    session_data.user_id,
                    &session_data.data,
                    to_cql_ts(session_data.expires_at),
                    to_cql_ts(session_data.created_at),
                ),
            )
            .await
            .context("Failed to insert session")?;

        info!(
            "Session saved: {} for user {}",
            session_data.session_id, session_data.user_id
        );
        Ok(())
    }

    /// Delete a session by ID.
    pub async fn delete_session(&self, session_id: Uuid) -> Result<()> {
        debug!("Deleting session {}", session_id);

        self.session
            .session()
            .execute_unpaged(&self.ps_delete, (session_id,))
            .await
            .context("Failed to delete session")?;

        info!("Session deleted: {}", session_id);
        Ok(())
    }
}
