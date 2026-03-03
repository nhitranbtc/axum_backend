use async_trait::async_trait;
use scylla::statement::prepared::PreparedStatement;
use scylla::value::CqlTimestamp;
use std::sync::Arc;
use tracing::debug;
use uuid::Uuid;

use crate::{
    domain::{
        entities::User,
        repositories::user_repository::{RepositoryError, UserRepository},
        value_objects::{Email, UserId, UserRole},
    },
    infrastructure::database::scylla::{
        connection::ScyllaSession,
        models::UserRow,
        scylla_utils::{from_cql_ts, opt_from_cql_ts, opt_to_cql_ts, to_cql_ts, UserRowTuple},
    },
};

/// ScyllaDB implementation of `UserRepository`.
///
/// All queries are prepared at construction time; ScyllaDB caches their plan server-side
/// so subsequent executions skip the parse/plan phase entirely.
#[derive(Clone)]
pub struct RepositoryImpl {
    session: Arc<ScyllaSession>,
    ps_upsert: PreparedStatement,
    ps_find_by_id: PreparedStatement,
    ps_find_by_email: PreparedStatement,
    ps_count: PreparedStatement,
    ps_list: PreparedStatement,
    ps_delete: PreparedStatement,
    ps_delete_all: PreparedStatement,
}

impl RepositoryImpl {
    pub async fn new(session: Arc<ScyllaSession>) -> Result<Self, RepositoryError> {
        let s = session.session();
        let prepare = |q: &'static str| async move {
            s.prepare(q)
                .await
                .map_err(|e| RepositoryError::Internal(e.to_string()))
        };

        let (ps_upsert, ps_find_by_id, ps_find_by_email, ps_count, ps_list, ps_delete, ps_delete_all) = tokio::try_join!(
            prepare(
                "INSERT INTO users (user_id, email, name, password_hash, role, is_active, \
                 email_verified, confirmation_code, confirmation_code_expires_at, \
                 last_login, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
            ),
            prepare(
                "SELECT user_id, email, name, password_hash, role, is_active, \
                 email_verified, confirmation_code, confirmation_code_expires_at, \
                 last_login, created_at, updated_at FROM users WHERE user_id = ?"
            ),
            prepare(
                "SELECT user_id, email, name, password_hash, role, is_active, \
                 email_verified, confirmation_code, confirmation_code_expires_at, \
                 last_login, created_at, updated_at FROM users WHERE email = ? ALLOW FILTERING"
            ),
            prepare("SELECT COUNT(*) FROM users"),
            prepare(
                "SELECT user_id, email, name, password_hash, role, is_active, \
                 email_verified, confirmation_code, confirmation_code_expires_at, \
                 last_login, created_at, updated_at FROM users LIMIT ?"
            ),
            prepare("DELETE FROM users WHERE user_id = ?"),
            prepare("TRUNCATE users"),
        )?;

        Ok(Self {
            session,
            ps_upsert,
            ps_find_by_id,
            ps_find_by_email,
            ps_count,
            ps_list,
            ps_delete,
            ps_delete_all,
        })
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn scylla_err(e: impl std::fmt::Display) -> RepositoryError {
        RepositoryError::Internal(e.to_string())
    }

    fn row_to_user_row(row: UserRowTuple) -> UserRow {
        UserRow {
            user_id: row.0,
            email: row.1,
            name: row.2,
            password_hash: row.3,
            role: row.4,
            is_active: row.5,
            email_verified: row.6,
            confirmation_code: row.7,
            confirmation_code_expires_at: opt_from_cql_ts(row.8),
            last_login: opt_from_cql_ts(row.9),
            created_at: from_cql_ts(row.10),
            updated_at: from_cql_ts(row.11),
        }
    }

    fn user_row_to_entity(row: UserRow) -> Result<User, RepositoryError> {
        Ok(User::from_existing(
            UserId::from_uuid(row.user_id),
            Email::parse(&row.email).map_err(|e| {
                RepositoryError::Internal(format!("Invalid email in DB: {}", e))
            })?,
            row.name,
            row.password_hash,
            UserRole::parse(&row.role).unwrap_or_default(),
            row.is_active,
            row.email_verified,
            row.confirmation_code,
            row.confirmation_code_expires_at,
            row.last_login,
            row.created_at,
            row.updated_at,
        ))
    }

    /// Upsert a user (used by both `save` and `update`).
    async fn upsert_user(&self, user: &User) -> Result<(), RepositoryError> {
        self.session
            .session()
            .execute_unpaged(
                &self.ps_upsert,
                (
                    *user.id.as_uuid(),
                    user.email.as_str(),
                    user.name.as_str(),
                    &user.password_hash,
                    user.role.to_string(),
                    user.is_active,
                    user.is_email_verified,
                    &user.confirmation_code,
                    opt_to_cql_ts(user.confirmation_code_expires_at),
                    opt_to_cql_ts(user.last_login),
                    to_cql_ts(user.created_at),
                    to_cql_ts(user.updated_at),
                ),
            )
            .await
            .map_err(Self::scylla_err)?;
        Ok(())
    }

    /// Fetch a user by primary key.
    async fn fetch_by_id(&self, id: Uuid) -> Result<Option<UserRow>, RepositoryError> {
        let result = self
            .session
            .session()
            .execute_unpaged(&self.ps_find_by_id, (id,))
            .await
            .map_err(Self::scylla_err)?;

        let rows_iter = result
            .into_rows_result()
            .map_err(|e| RepositoryError::Internal(e.to_string()))?;

        if let Some(row) = rows_iter
            .rows::<UserRowTuple>()
            .map_err(|e| RepositoryError::Internal(e.to_string()))?
            .next()
        {
            let row = row.map_err(|e| RepositoryError::Internal(e.to_string()))?;
            Ok(Some(Self::row_to_user_row(row)))
        } else {
            Ok(None)
        }
    }

    /// Fetch a user via the email secondary index.
    async fn fetch_by_email_str(&self, email: &str) -> Result<Option<UserRow>, RepositoryError> {
        let result = self
            .session
            .session()
            .execute_unpaged(&self.ps_find_by_email, (email,))
            .await
            .map_err(Self::scylla_err)?;

        let rows_iter = result
            .into_rows_result()
            .map_err(|e| RepositoryError::Internal(e.to_string()))?;

        if let Some(row) = rows_iter
            .rows::<UserRowTuple>()
            .map_err(|e| RepositoryError::Internal(e.to_string()))?
            .next()
        {
            let row = row.map_err(|e| RepositoryError::Internal(e.to_string()))?;
            Ok(Some(Self::row_to_user_row(row)))
        } else {
            Ok(None)
        }
    }
}

#[async_trait]
impl UserRepository for RepositoryImpl {
    async fn save(&self, user: &User) -> Result<User, RepositoryError> {
        debug!("Saving user {}", user.id.as_uuid());
        self.upsert_user(user).await?;
        Ok(user.clone())
    }

    async fn update(&self, user: &User) -> Result<User, RepositoryError> {
        debug!("Updating user {}", user.id.as_uuid());
        self.upsert_user(user).await?;
        Ok(user.clone())
    }

    async fn find_by_id(&self, id: UserId) -> Result<Option<User>, RepositoryError> {
        debug!("Finding user by id {}", id.as_uuid());
        let row = self.fetch_by_id(*id.as_uuid()).await?;
        row.map(Self::user_row_to_entity).transpose()
    }

    async fn find_by_email(&self, email: &Email) -> Result<Option<User>, RepositoryError> {
        debug!("Finding user by email {}", email.as_str());
        let row = self.fetch_by_email_str(email.as_str()).await?;
        row.map(Self::user_row_to_entity).transpose()
    }

    async fn exists_by_email(&self, email: &Email) -> Result<bool, RepositoryError> {
        Ok(self.fetch_by_email_str(email.as_str()).await?.is_some())
    }

    async fn count(&self) -> Result<i64, RepositoryError> {
        // Note: scalar COUNT on a wide table is fine but expensive; consider a counter table
        // for hot-path code.
        let result = self
            .session
            .session()
            .execute_unpaged(&self.ps_count, &[])
            .await
            .map_err(Self::scylla_err)?;

        let rows_iter = result
            .into_rows_result()
            .map_err(|e| RepositoryError::Internal(e.to_string()))?;

        let count: i64 = rows_iter
            .rows::<(i64,)>()
            .map_err(|e| RepositoryError::Internal(e.to_string()))?
            .next()
            .ok_or_else(|| RepositoryError::Internal("No COUNT result".to_string()))?
            .map(|(c,)| c)
            .map_err(|e| RepositoryError::Internal(e.to_string()))?;

        Ok(count)
    }

    async fn list_paginated(
        &self,
        limit: i64,
        _offset: i64,
    ) -> Result<Vec<User>, RepositoryError> {
        // ScyllaDB pagination uses paging state, not OFFSET.
        // We use a simple LIMIT here; callers that need cursor-based pagination
        // should use the raw session paging API.
        let result = self
            .session
            .session()
            .execute_unpaged(&self.ps_list, (limit as i32,))
            .await
            .map_err(Self::scylla_err)?;

        let rows_iter = result
            .into_rows_result()
            .map_err(|e| RepositoryError::Internal(e.to_string()))?;

        let mut users = Vec::new();
        for row in rows_iter
            .rows::<UserRowTuple>()
            .map_err(|e| RepositoryError::Internal(e.to_string()))?
        {
            let row = row.map_err(|e| RepositoryError::Internal(e.to_string()))?;
            let user = Self::user_row_to_entity(Self::row_to_user_row(row))?;
            users.push(user);
        }

        Ok(users)
    }

    async fn delete(&self, id: UserId) -> Result<bool, RepositoryError> {
        debug!("Deleting user {}", id.as_uuid());
        if self.fetch_by_id(*id.as_uuid()).await?.is_none() {
            return Ok(false);
        }
        self.session
            .session()
            .execute_unpaged(&self.ps_delete, (*id.as_uuid(),))
            .await
            .map_err(Self::scylla_err)?;
        Ok(true)
    }

    async fn delete_all(&self) -> Result<usize, RepositoryError> {
        self.session
            .session()
            .execute_unpaged(&self.ps_delete_all, &[])
            .await
            .map_err(Self::scylla_err)?;
        Ok(0) // TRUNCATE doesn't return a row count
    }
}
