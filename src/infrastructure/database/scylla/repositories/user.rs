use async_trait::async_trait;
use scylla::client::caching_session::CachingSession;
use std::sync::Arc;
use tracing::debug;
use uuid::Uuid;

use crate::domain::{
    entities::User,
    repositories::user_repository::{RepositoryError, UserRepository},
    value_objects::{Email, UserId, UserRole},
};
use crate::infrastructure::database::scylla::{
    connection::ScyllaSession,
    models::UserRow,
    operations::prelude::*,
};

/// ScyllaDB implementation of [`UserRepository`].
///
/// Holds only an `Arc<CachingSession>` — no `PreparedStatement` fields.
/// The `CachingSession` transparently prepares and caches each unique query
/// string the first time it is executed (charybdis-style architecture).
#[derive(Clone)]
pub struct RepositoryImpl {
    session: Arc<CachingSession>,
}

impl RepositoryImpl {
    pub fn new(session: Arc<ScyllaSession>) -> Self {
        // ScyllaSession is a thin newtype around Arc<CachingSession>.
        // We clone the inner session so the repo can be freely shared.
        Self { session: session.session() }
    }

    // ── Private helpers ───────────────────────────────────────────────────────

    fn db_err(e: impl std::fmt::Display) -> RepositoryError {
        RepositoryError::Internal(e.to_string())
    }

    fn row_to_entity(row: UserRow) -> Result<User, RepositoryError> {
        Ok(User::from_existing(
            UserId::from_uuid(row.user_id),
            Email::parse(&row.email)
                .map_err(|e| RepositoryError::Internal(format!("Invalid email in DB: {e}")))?,
            row.name,
            row.password_hash,
            UserRole::parse(&row.role).unwrap_or_default(),
            row.is_active,
            row.email_verified,
            row.confirmation_code,
            UserRow::from_opt_ts(row.confirmation_code_expires_at),
            UserRow::from_opt_ts(row.last_login),
            UserRow::from_ts(row.created_at),
            UserRow::from_ts(row.updated_at),
        ))
    }

    /// INSERT or full-replace a user row.
    async fn upsert(&self, user: &User) -> Result<(), RepositoryError> {
        let row = UserRow {
            user_id: *user.id.as_uuid(),
            email: user.email.as_str().to_string(),
            name: user.name.clone(),
            password_hash: user.password_hash.clone(),
            role: user.role.to_string(),
            is_active: user.is_active,
            email_verified: user.is_email_verified,
            confirmation_code: user.confirmation_code.clone(),
            confirmation_code_expires_at: UserRow::opt_ts(user.confirmation_code_expires_at),
            last_login: UserRow::opt_ts(user.last_login),
            created_at: UserRow::ts(user.created_at),
            updated_at: UserRow::ts(user.updated_at),
        };

        row.insert()
            .execute(&self.session)
            .await
            .map_err(Self::db_err)?;
        Ok(())
    }

    async fn fetch_by_id(&self, id: Uuid) -> Result<Option<UserRow>, RepositoryError> {
        UserRow::maybe_find_by_primary_key_value((id,))
            .execute(&self.session)
            .await
            .map_err(Self::db_err)
    }

    async fn fetch_by_email(&self, email: &str) -> Result<Option<UserRow>, RepositoryError> {
        UserRow::maybe_find_first(UserRow::FIND_BY_EMAIL_QUERY, (email,))
            .execute(&self.session)
            .await
            .map_err(Self::db_err)
    }
}

#[async_trait]
impl UserRepository for RepositoryImpl {
    async fn save(&self, user: &User) -> Result<User, RepositoryError> {
        debug!("UserRepo::save {}", user.id.as_uuid());
        self.upsert(user).await?;
        Ok(user.clone())
    }

    async fn update(&self, user: &User) -> Result<User, RepositoryError> {
        debug!("UserRepo::update {}", user.id.as_uuid());
        self.upsert(user).await?;
        Ok(user.clone())
    }

    async fn find_by_id(&self, id: UserId) -> Result<Option<User>, RepositoryError> {
        debug!("UserRepo::find_by_id {}", id.as_uuid());
        self.fetch_by_id(*id.as_uuid())
            .await?
            .map(Self::row_to_entity)
            .transpose()
    }

    async fn find_by_email(&self, email: &Email) -> Result<Option<User>, RepositoryError> {
        debug!("UserRepo::find_by_email {}", email.as_str());
        self.fetch_by_email(email.as_str())
            .await?
            .map(Self::row_to_entity)
            .transpose()
    }

    async fn exists_by_email(&self, email: &Email) -> Result<bool, RepositoryError> {
        Ok(self.fetch_by_email(email.as_str()).await?.is_some())
    }

    async fn count(&self) -> Result<i64, RepositoryError> {
        // Note: full-table COUNT is expensive; consider a dedicated counter table
        // for hot-path code.
        let result = execute_unpaged(&self.session, UserRow::COUNT_QUERY, &[])
            .await
            .map_err(Self::db_err)?
            .into_rows_result()
            .map_err(|e| RepositoryError::Internal(e.to_string()))?;

        let (count,): (i64,) = result
            .first_row()
            .map_err(|e| RepositoryError::Internal(e.to_string()))?;
        Ok(count)
    }

    async fn list_paginated(
        &self,
        limit: i64,
        _offset: i64,
    ) -> Result<Vec<User>, RepositoryError> {
        // ScyllaDB uses paging state, not OFFSET. A simple LIMIT is used here;
        // callers needing cursor pagination should use execute_single_page directly.
        let rows = UserRow::find(UserRow::FIND_ALL_QUERY, (limit as i32,))
            .execute(&self.session)
            .await
            .map_err(Self::db_err)?;

        Ok(rows.into_iter().flat_map(Self::row_to_entity).collect())
    }

    async fn delete(&self, id: UserId) -> Result<bool, RepositoryError> {
        debug!("UserRepo::delete {}", id.as_uuid());
        if let Some(row) = self.fetch_by_id(*id.as_uuid()).await? {
            row.delete().execute(&self.session).await.map_err(Self::db_err)?;
            return Ok(true);
        }
        Ok(false)
    }

    async fn delete_all(&self) -> Result<usize, RepositoryError> {
        execute_unpaged(&self.session, UserRow::DELETE_ALL_QUERY, &[])
            .await
            .map_err(Self::db_err)?;
        Ok(0) // TRUNCATE does not return a row count
    }
}
