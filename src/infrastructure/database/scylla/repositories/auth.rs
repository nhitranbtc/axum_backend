use async_trait::async_trait;
use scylla::statement::prepared::PreparedStatement;
use scylla::value::CqlTimestamp;
use std::sync::Arc;
use tracing::debug;
use uuid::Uuid;

use crate::{
    domain::{
        entities::{refresh_token::RefreshToken, User},
        repositories::{AuthRepository, AuthRepositoryError},
        value_objects::{Email, UserId, UserRole},
    },
    infrastructure::database::scylla::{
        connection::ScyllaSession,
        scylla_utils::{from_cql_ts, opt_from_cql_ts, opt_to_cql_ts, to_cql_ts, UserRowTuple},
    },
};

/// ScyllaDB implementation of `AuthRepository`.
///
/// All queries are prepared at construction time so ScyllaDB caches their plan server-side.
pub struct RepositoryImpl {
    session: Arc<ScyllaSession>,
    ps_find_user_by_email: PreparedStatement,
    ps_insert_user: PreparedStatement,
    ps_update_last_login: PreparedStatement,
    ps_update_user: PreparedStatement,
    ps_insert_token: PreparedStatement,
    ps_find_token: PreparedStatement,
    ps_revoke_token: PreparedStatement,
    ps_tokens_by_user: PreparedStatement,
}

impl RepositoryImpl {
    pub async fn new(session: Arc<ScyllaSession>) -> Result<Self, AuthRepositoryError> {
        let db_err =
            |e: scylla::errors::PrepareError| AuthRepositoryError::DatabaseError(e.to_string());
        let s = session.session();

        let (
            ps_find_user_by_email,
            ps_insert_user,
            ps_update_last_login,
            ps_update_user,
            ps_insert_token,
            ps_find_token,
            ps_revoke_token,
            ps_tokens_by_user,
        ) = tokio::try_join!(
            async { s.prepare(
                "SELECT user_id, email, name, password_hash, role, is_active, \
                 email_verified, confirmation_code, confirmation_code_expires_at, \
                 last_login, created_at, updated_at FROM users WHERE email = ? ALLOW FILTERING"
            ).await.map_err(db_err) },
            async { s.prepare(
                "INSERT INTO users (user_id, email, name, password_hash, role, is_active, \
                 email_verified, confirmation_code, confirmation_code_expires_at, \
                 last_login, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
            ).await.map_err(db_err) },
            async { s.prepare(
                "UPDATE users SET last_login = ?, updated_at = ? WHERE user_id = ?"
            ).await.map_err(db_err) },
            async { s.prepare(
                "UPDATE users SET name = ?, email = ?, password_hash = ?, role = ?, \
                 is_active = ?, email_verified = ?, confirmation_code = ?, \
                 confirmation_code_expires_at = ?, updated_at = ? WHERE user_id = ?"
            ).await.map_err(db_err) },
            async { s.prepare(
                "INSERT INTO refresh_tokens (token_hash, user_id, expires_at, created_at, revoked_at) \
                 VALUES (?, ?, ?, ?, ?)"
            ).await.map_err(db_err) },
            async { s.prepare(
                "SELECT token_hash, user_id, expires_at, created_at, revoked_at \
                 FROM refresh_tokens WHERE token_hash = ?"
            ).await.map_err(db_err) },
            async { s.prepare(
                "UPDATE refresh_tokens SET revoked_at = ? WHERE token_hash = ?"
            ).await.map_err(db_err) },
            async { s.prepare(
                "SELECT token_hash FROM refresh_tokens WHERE user_id = ? ALLOW FILTERING"
            ).await.map_err(db_err) },
        )?;

        Ok(Self {
            session,
            ps_find_user_by_email,
            ps_insert_user,
            ps_update_last_login,
            ps_update_user,
            ps_insert_token,
            ps_find_token,
            ps_revoke_token,
            ps_tokens_by_user,
        })
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn db_err(e: impl std::fmt::Display) -> AuthRepositoryError {
        AuthRepositoryError::DatabaseError(e.to_string())
    }

    // ── Private fetch helpers ─────────────────────────────────────────────────

    async fn query_user_by_email(
        &self,
        email: &str,
    ) -> Result<Option<User>, AuthRepositoryError> {
        let result = self
            .session
            .session()
            .execute_unpaged(&self.ps_find_user_by_email, (email,))
            .await
            .map_err(Self::db_err)?;

        let rows_iter = result
            .into_rows_result()
            .map_err(|e| AuthRepositoryError::DatabaseError(e.to_string()))?;

        let maybe_row = rows_iter
            .rows::<UserRowTuple>()
            .map_err(|e| AuthRepositoryError::DatabaseError(e.to_string()))?
            .next();

        let Some(row) = maybe_row else {
            return Ok(None);
        };
        let (
            user_id, email_col, name, password_hash, role, is_active,
            email_verified, confirmation_code, cc_expires, last_login,
            created_at, updated_at,
        ) = row.map_err(|e| AuthRepositoryError::DatabaseError(e.to_string()))?;

        Ok(Some(User::from_existing(
            UserId::from_uuid(user_id),
            Email::parse(&email_col).unwrap_or_else(|_| {
                panic!("Invalid email stored in database: {}", email_col)
            }),
            name,
            password_hash,
            UserRole::parse(&role).unwrap_or_default(),
            is_active,
            email_verified,
            confirmation_code,
            opt_from_cql_ts(cc_expires),
            opt_from_cql_ts(last_login),
            from_cql_ts(created_at),
            from_cql_ts(updated_at),
        )))
    }

    async fn query_token_by_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<RefreshToken>, AuthRepositoryError> {
        let result = self
            .session
            .session()
            .execute_unpaged(&self.ps_find_token, (token_hash,))
            .await
            .map_err(Self::db_err)?;

        let rows_iter = result
            .into_rows_result()
            .map_err(|e| AuthRepositoryError::DatabaseError(e.to_string()))?;

        let maybe_row = rows_iter
            .rows::<(String, Uuid, CqlTimestamp, CqlTimestamp, Option<CqlTimestamp>)>()
            .map_err(|e| AuthRepositoryError::DatabaseError(e.to_string()))?
            .next();

        let Some(row) = maybe_row else {
            return Ok(None);
        };
        let (hash, user_id, expires_at, created_at, revoked_at) =
            row.map_err(|e| AuthRepositoryError::DatabaseError(e.to_string()))?;

        Ok(Some(RefreshToken {
            id: Uuid::new_v4(),
            user_id,
            token_hash: hash,
            expires_at: from_cql_ts(expires_at),
            created_at: from_cql_ts(created_at),
            revoked_at: opt_from_cql_ts(revoked_at),
        }))
    }
}

#[async_trait]
impl AuthRepository for RepositoryImpl {
    async fn find_by_email(
        &self,
        email: &str,
    ) -> Result<Option<User>, AuthRepositoryError> {
        debug!("AuthRepo: find_by_email {}", email);
        self.query_user_by_email(email).await
    }

    async fn create_user(
        &self,
        email: &str,
        name: &str,
        password_hash: Option<String>,
        confirmation_code: Option<String>,
        expires_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<User, AuthRepositoryError> {
        debug!("AuthRepo: create_user {}", email);

        if self.query_user_by_email(email).await?.is_some() {
            return Err(AuthRepositoryError::EmailAlreadyExists);
        }

        let id = Uuid::new_v4();
        let now = chrono::Utc::now();

        self.session
            .session()
            .execute_unpaged(
                &self.ps_insert_user,
                (
                    id, email, name, &password_hash,
                    "viewer", false, false,
                    &confirmation_code,
                    opt_to_cql_ts(expires_at),
                    None::<CqlTimestamp>,
                    to_cql_ts(now),
                    to_cql_ts(now),
                ),
            )
            .await
            .map_err(Self::db_err)?;

        Ok(User::from_existing(
            UserId::from_uuid(id),
            Email::parse(email).unwrap(),
            name.to_string(),
            password_hash,
            UserRole::default(),
            false, false,
            confirmation_code,
            expires_at,
            None,
            now, now,
        ))
    }

    async fn update_last_login(&self, user_id: Uuid) -> Result<(), AuthRepositoryError> {
        debug!("AuthRepo: update_last_login {}", user_id);
        let now = chrono::Utc::now();
        self.session
            .session()
            .execute_unpaged(
                &self.ps_update_last_login,
                (to_cql_ts(now), to_cql_ts(now), user_id),
            )
            .await
            .map_err(Self::db_err)?;
        Ok(())
    }

    async fn update_user(&self, user: &User) -> Result<User, AuthRepositoryError> {
        debug!("AuthRepo: update_user {}", user.id.as_uuid());
        let now = chrono::Utc::now();
        self.session
            .session()
            .execute_unpaged(
                &self.ps_update_user,
                (
                    user.name.as_str(),
                    user.email.as_str(),
                    &user.password_hash,
                    user.role.to_string(),
                    user.is_active,
                    user.is_email_verified,
                    &user.confirmation_code,
                    opt_to_cql_ts(user.confirmation_code_expires_at),
                    to_cql_ts(now),
                    *user.id.as_uuid(),
                ),
            )
            .await
            .map_err(Self::db_err)?;
        let mut updated = user.clone();
        updated.updated_at = now;
        Ok(updated)
    }

    async fn save_refresh_token(
        &self,
        token: &RefreshToken,
    ) -> Result<(), AuthRepositoryError> {
        debug!("AuthRepo: save_refresh_token for user {}", token.user_id);
        self.session
            .session()
            .execute_unpaged(
                &self.ps_insert_token,
                (
                    token.token_hash.as_str(),
                    token.user_id,
                    to_cql_ts(token.expires_at),
                    to_cql_ts(token.created_at),
                    opt_to_cql_ts(token.revoked_at),
                ),
            )
            .await
            .map_err(Self::db_err)?;
        Ok(())
    }

    async fn find_refresh_token(
        &self,
        token_hash: &str,
    ) -> Result<Option<RefreshToken>, AuthRepositoryError> {
        debug!("AuthRepo: find_refresh_token");
        self.query_token_by_hash(token_hash).await
    }

    async fn revoke_refresh_token(
        &self,
        token_hash: &str,
    ) -> Result<(), AuthRepositoryError> {
        debug!("AuthRepo: revoke_refresh_token");
        match self.query_token_by_hash(token_hash).await? {
            None => return Err(AuthRepositoryError::TokenNotFound),
            Some(t) if t.revoked_at.is_some() => {
                return Err(AuthRepositoryError::TokenNotFound)
            }
            _ => {}
        }
        let now = chrono::Utc::now();
        self.session
            .session()
            .execute_unpaged(&self.ps_revoke_token, (to_cql_ts(now), token_hash))
            .await
            .map_err(Self::db_err)?;
        Ok(())
    }

    async fn revoke_all_user_tokens(
        &self,
        user_id: Uuid,
    ) -> Result<(), AuthRepositoryError> {
        debug!("AuthRepo: revoke_all_user_tokens {}", user_id);

        let result = self
            .session
            .session()
            .execute_unpaged(&self.ps_tokens_by_user, (user_id,))
            .await
            .map_err(Self::db_err)?;

        let rows_iter = result
            .into_rows_result()
            .map_err(|e| AuthRepositoryError::DatabaseError(e.to_string()))?;

        let now_ts = to_cql_ts(chrono::Utc::now());

        for row in rows_iter
            .rows::<(String,)>()
            .map_err(|e| AuthRepositoryError::DatabaseError(e.to_string()))?
        {
            let (hash,) = row.map_err(|e| AuthRepositoryError::DatabaseError(e.to_string()))?;
            self.session
                .session()
                .execute_unpaged(&self.ps_revoke_token, (now_ts, hash.as_str()))
                .await
                .map_err(Self::db_err)?;
        }
        Ok(())
    }

    async fn cleanup_expired_tokens(&self) -> Result<u64, AuthRepositoryError> {
        // ScyllaDB doesn't efficiently support DELETE … WHERE expires_at < now()
        // on a secondary-indexed column without ALLOW FILTERING.
        // Use TTL on the table or a dedicated cleanup job in production.
        Ok(0)
    }
}
