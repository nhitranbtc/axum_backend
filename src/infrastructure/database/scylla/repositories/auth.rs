use async_trait::async_trait;
use scylla::client::caching_session::CachingSession;
use std::sync::Arc;
use tracing::debug;
use uuid::Uuid;

use crate::domain::{
    entities::{refresh_token::RefreshToken, User},
    repositories::{AuthRepository, AuthRepositoryError},
    value_objects::{Email, UserId, UserRole},
};
use crate::infrastructure::database::scylla::{
    connection::ScyllaSession,
    models::{RefreshTokenRow, UserRow},
    operations::prelude::*,
};

/// ScyllaDB implementation of [`AuthRepository`].
///
/// Holds only an `Arc<CachingSession>` — no `PreparedStatement` fields.
/// The `CachingSession` automatically prepares and caches each query string
/// the first time it is executed (charybdis-style architecture).
pub struct RepositoryImpl {
    session: Arc<CachingSession>,
}

impl RepositoryImpl {
    pub fn new(session: Arc<ScyllaSession>) -> Self {
        Self { session: session.session() }
    }

    // ── Private helpers ───────────────────────────────────────────────────────

    fn db_err(e: impl std::fmt::Display) -> AuthRepositoryError {
        AuthRepositoryError::DatabaseError(e.to_string())
    }

    fn user_row_to_entity(row: UserRow) -> Result<User, AuthRepositoryError> {
        Ok(User::from_existing(
            UserId::from_uuid(row.user_id),
            Email::parse(&row.email)
                .unwrap_or_else(|_| panic!("Invalid email stored in database: {}", row.email)),
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

    async fn query_user_by_email(&self, email: &str) -> Result<Option<User>, AuthRepositoryError> {
        UserRow::maybe_find_first(UserRow::FIND_BY_EMAIL_QUERY, (email,))
            .execute(&self.session)
            .await
            .map_err(Self::db_err)?
            .map(Self::user_row_to_entity)
            .transpose()
    }

    async fn query_token_by_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<RefreshToken>, AuthRepositoryError> {
        let row_opt = RefreshTokenRow::maybe_find_by_primary_key_value((token_hash.to_string(),))
            .execute(&self.session)
            .await
            .map_err(Self::db_err)?;

        Ok(row_opt.map(|row| RefreshToken {
            id: Uuid::new_v4(),
            user_id: row.user_id,
            token_hash: row.token_hash,
            expires_at: RefreshTokenRow::from_ts(row.expires_at),
            created_at: RefreshTokenRow::from_ts(row.created_at),
            revoked_at: RefreshTokenRow::from_opt_ts(row.revoked_at),
        }))
    }
}

#[async_trait]
impl AuthRepository for RepositoryImpl {
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AuthRepositoryError> {
        debug!("AuthRepo::find_by_email {}", email);
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
        debug!("AuthRepo::create_user {}", email);

        if self.query_user_by_email(email).await?.is_some() {
            return Err(AuthRepositoryError::EmailAlreadyExists);
        }

        let id = Uuid::new_v4();
        let now = chrono::Utc::now();

        let row = UserRow {
            user_id: id,
            email: email.to_string(),
            name: name.to_string(),
            password_hash: password_hash.clone(),
            role: "viewer".to_string(),
            is_active: false,
            email_verified: false,
            confirmation_code: confirmation_code.clone(),
            confirmation_code_expires_at: UserRow::opt_ts(expires_at),
            last_login: None,
            created_at: UserRow::ts(now),
            updated_at: UserRow::ts(now),
        };

        row.insert().execute(&self.session).await.map_err(Self::db_err)?;

        Ok(User::from_existing(
            UserId::from_uuid(id),
            Email::parse(email).unwrap(),
            name.to_string(),
            password_hash,
            UserRole::default(),
            false,
            false,
            confirmation_code,
            expires_at,
            None,
            now,
            now,
        ))
    }

    async fn update_last_login(&self, user_id: Uuid) -> Result<(), AuthRepositoryError> {
        debug!("AuthRepo::update_last_login {}", user_id);
        let now = chrono::Utc::now();
        execute_unpaged(
            &self.session,
            UserRow::UPDATE_LAST_LOGIN_QUERY,
            (UserRow::ts(now), UserRow::ts(now), user_id),
        )
        .await
        .map_err(Self::db_err)?;
        Ok(())
    }

    async fn update_user(&self, user: &User) -> Result<User, AuthRepositoryError> {
        debug!("AuthRepo::update_user {}", user.id.as_uuid());
        let now = chrono::Utc::now();
        execute_unpaged(
            &self.session,
            UserRow::UPDATE_QUERY,
            (
                user.name.as_str(),
                user.email.as_str(),
                &user.password_hash,
                user.role.to_string(),
                user.is_active,
                user.is_email_verified,
                &user.confirmation_code,
                UserRow::opt_ts(user.confirmation_code_expires_at),
                UserRow::ts(now),
                *user.id.as_uuid(),
            ),
        )
        .await
        .map_err(Self::db_err)?;

        let mut updated = user.clone();
        updated.updated_at = now;
        Ok(updated)
    }

    async fn save_refresh_token(&self, token: &RefreshToken) -> Result<(), AuthRepositoryError> {
        debug!("AuthRepo::save_refresh_token for user {}", token.user_id);
        let row = RefreshTokenRow {
            token_hash: token.token_hash.clone(),
            user_id: token.user_id,
            expires_at: RefreshTokenRow::ts(token.expires_at),
            created_at: RefreshTokenRow::ts(token.created_at),
            revoked_at: RefreshTokenRow::opt_ts(token.revoked_at),
        };
        row.insert().execute(&self.session).await.map_err(Self::db_err)?;
        Ok(())
    }

    async fn find_refresh_token(
        &self,
        token_hash: &str,
    ) -> Result<Option<RefreshToken>, AuthRepositoryError> {
        debug!("AuthRepo::find_refresh_token");
        self.query_token_by_hash(token_hash).await
    }

    async fn revoke_refresh_token(&self, token_hash: &str) -> Result<(), AuthRepositoryError> {
        debug!("AuthRepo::revoke_refresh_token");
        match self.query_token_by_hash(token_hash).await? {
            None | Some(RefreshToken { revoked_at: Some(_), .. }) => {
                return Err(AuthRepositoryError::TokenNotFound)
            },
            _ => {},
        }
        execute_unpaged(
            &self.session,
            RefreshTokenRow::REVOKE_QUERY,
            (RefreshTokenRow::ts(chrono::Utc::now()), token_hash),
        )
        .await
        .map_err(Self::db_err)?;
        Ok(())
    }

    async fn revoke_all_user_tokens(&self, user_id: Uuid) -> Result<(), AuthRepositoryError> {
        debug!("AuthRepo::revoke_all_user_tokens {}", user_id);

        let result =
            execute_unpaged(&self.session, RefreshTokenRow::FIND_HASHES_BY_USER_QUERY, (user_id,))
                .await
                .map_err(Self::db_err)?
                .into_rows_result()
                .map_err(|e| AuthRepositoryError::DatabaseError(e.to_string()))?;

        let now_ts = RefreshTokenRow::ts(chrono::Utc::now());

        for row in result
            .rows::<(String,)>()
            .map_err(|e| AuthRepositoryError::DatabaseError(e.to_string()))?
        {
            let (hash,) = row.map_err(|e| AuthRepositoryError::DatabaseError(e.to_string()))?;
            execute_unpaged(&self.session, RefreshTokenRow::REVOKE_QUERY, (now_ts, hash.as_str()))
                .await
                .map_err(Self::db_err)?;
        }
        Ok(())
    }

    async fn cleanup_expired_tokens(&self) -> Result<u64, AuthRepositoryError> {
        // ScyllaDB doesn't efficiently support DELETE … WHERE expires_at < now()
        // on a secondary-indexed column without ALLOW FILTERING.
        // Use TTL on the table or a dedicated background cleanup job in production.
        Ok(0)
    }
}
