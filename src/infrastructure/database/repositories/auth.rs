use crate::{
    domain::{
        entities::{RefreshToken, User},
        repositories::{AuthRepository, AuthRepositoryError},
        value_objects::{Email, UserId, UserRole},
    },
    infrastructure::database::{
        models::{RefreshTokenModel, UserModel},
        schema::{refresh_tokens, users},
        DbPool,
    },
};
use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

/// PostgreSQL implementation of AuthRepository
///
/// This implementation uses Diesel ORM with async PostgreSQL.
/// The struct name is generic to avoid coupling to specific database technology.
pub struct RepositoryImpl {
    pool: DbPool,
}

impl RepositoryImpl {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// Helper: Convert UserModel to domain User entity
    fn user_model_to_entity(model: UserModel) -> User {
        User::from_existing(
            UserId::from_uuid(model.id),
            Email::parse(&model.email).unwrap_or_else(|_| {
                // This should never happen in production as emails are validated before insert
                panic!("Invalid email in database: {}", model.email)
            }),
            model.name,
            model.password_hash,
            UserRole::parse(&model.role).unwrap_or_default(),
            model.is_active,
            model.email_verified,
            model.confirmation_code,
            model.confirmation_code_expires_at,
            model.last_login,
            model.created_at,
            model.updated_at,
        )
    }

    /// Helper: Convert RefreshTokenModel to domain RefreshToken entity
    fn token_model_to_entity(model: RefreshTokenModel) -> RefreshToken {
        RefreshToken {
            id: model.id,
            user_id: model.user_id,
            token_hash: model.token_hash,
            expires_at: model.expires_at,
            created_at: model.created_at,
            revoked_at: model.revoked_at,
        }
    }

    /// Helper: Convert domain RefreshToken entity to RefreshTokenModel
    fn token_entity_to_model(token: &RefreshToken) -> RefreshTokenModel {
        RefreshTokenModel {
            id: token.id,
            user_id: token.user_id,
            token_hash: token.token_hash.clone(),
            expires_at: token.expires_at,
            created_at: token.created_at,
            revoked_at: token.revoked_at,
        }
    }
}

#[async_trait]
impl AuthRepository for RepositoryImpl {
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AuthRepositoryError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AuthRepositoryError::DatabaseError(e.to_string()))?;

        let result = users::table
            .filter(users::email.eq(email))
            .first::<UserModel>(&mut conn)
            .await
            .optional()
            .map_err(|e| AuthRepositoryError::DatabaseError(e.to_string()))?;

        Ok(result.map(Self::user_model_to_entity))
    }

    async fn create_user(
        &self,
        email: &str,
        name: &str,
        password_hash: Option<String>,
        confirmation_code: Option<String>,
        expires_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<User, AuthRepositoryError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AuthRepositoryError::DatabaseError(e.to_string()))?;

        let id = Uuid::new_v4();
        let now = chrono::Utc::now();

        let new_user = UserModel {
            id,
            email: email.to_string(),
            name: name.to_string(),
            created_at: now,
            updated_at: now,
            password_hash: password_hash.clone(), // Clone if needed or passed by value
            role: "viewer".to_string(),
            is_active: false, // Default inactive
            last_login: None,
            confirmation_code: confirmation_code.clone(),
            confirmation_code_expires_at: expires_at,
            email_verified: false,
        };

        diesel::insert_into(users::table)
            .values(&new_user)
            .execute(&mut conn)
            .await
            .map_err(|e| {
                if e.to_string().contains("duplicate key")
                    || e.to_string().contains("unique constraint")
                {
                    AuthRepositoryError::EmailAlreadyExists
                } else {
                    AuthRepositoryError::DatabaseError(e.to_string())
                }
            })?;

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
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AuthRepositoryError::DatabaseError(e.to_string()))?;

        let now = chrono::Utc::now();

        diesel::update(users::table.filter(users::id.eq(user_id)))
            .set((users::last_login.eq(now), users::updated_at.eq(now)))
            .execute(&mut conn)
            .await
            .map_err(|e| AuthRepositoryError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn update_user(&self, user: &User) -> Result<User, AuthRepositoryError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AuthRepositoryError::DatabaseError(e.to_string()))?;

        let now = chrono::Utc::now();
        let uid = user.id.as_uuid();

        diesel::update(users::table.filter(users::id.eq(uid)))
            .set((
                users::name.eq(&user.name),
                users::email.eq(user.email.as_str()),
                users::password_hash.eq(&user.password_hash),
                users::role.eq(user.role.to_string()),
                users::is_active.eq(user.is_active),
                users::email_verified.eq(user.is_email_verified),
                users::confirmation_code.eq(&user.confirmation_code),
                users::confirmation_code_expires_at.eq(user.confirmation_code_expires_at),
                users::updated_at.eq(now),
            ))
            .execute(&mut conn)
            .await
            .map_err(|e| AuthRepositoryError::DatabaseError(e.to_string()))?;

        // Return updated user (we already have it in memory mostly, but good to return consistent state)
        // For simplicity, return the input user with updated_at (or just fetch again if we want DB truth).
        // Since we updating, we can just return a clone with updated timestamp or fetch.
        // Returning modified clone is cheaper.
        let mut updated_user = user.clone();
        updated_user.updated_at = now;
        Ok(updated_user)
    }

    async fn save_refresh_token(&self, token: &RefreshToken) -> Result<(), AuthRepositoryError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AuthRepositoryError::DatabaseError(e.to_string()))?;

        let db_token = Self::token_entity_to_model(token);

        diesel::insert_into(refresh_tokens::table)
            .values(&db_token)
            .execute(&mut conn)
            .await
            .map_err(|e| AuthRepositoryError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn find_refresh_token(
        &self,
        token_hash: &str,
    ) -> Result<Option<RefreshToken>, AuthRepositoryError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AuthRepositoryError::DatabaseError(e.to_string()))?;

        let result = refresh_tokens::table
            .filter(refresh_tokens::token_hash.eq(token_hash))
            .first::<RefreshTokenModel>(&mut conn)
            .await
            .optional()
            .map_err(|e| AuthRepositoryError::DatabaseError(e.to_string()))?;

        Ok(result.map(Self::token_model_to_entity))
    }

    async fn revoke_refresh_token(&self, token_hash: &str) -> Result<(), AuthRepositoryError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AuthRepositoryError::DatabaseError(e.to_string()))?;

        let now = chrono::Utc::now();

        let rows_affected = diesel::update(
            refresh_tokens::table
                .filter(refresh_tokens::token_hash.eq(token_hash))
                .filter(refresh_tokens::revoked_at.is_null()),
        )
        .set(refresh_tokens::revoked_at.eq(now))
        .execute(&mut conn)
        .await
        .map_err(|e| AuthRepositoryError::DatabaseError(e.to_string()))?;

        if rows_affected == 0 {
            return Err(AuthRepositoryError::TokenNotFound);
        }

        Ok(())
    }

    async fn revoke_all_user_tokens(&self, user_id: Uuid) -> Result<(), AuthRepositoryError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AuthRepositoryError::DatabaseError(e.to_string()))?;

        let now = chrono::Utc::now();

        diesel::update(
            refresh_tokens::table
                .filter(refresh_tokens::user_id.eq(user_id))
                .filter(refresh_tokens::revoked_at.is_null()),
        )
        .set(refresh_tokens::revoked_at.eq(now))
        .execute(&mut conn)
        .await
        .map_err(|e| AuthRepositoryError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn cleanup_expired_tokens(&self) -> Result<u64, AuthRepositoryError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| AuthRepositoryError::DatabaseError(e.to_string()))?;

        let now = chrono::Utc::now();

        let rows_affected = diesel::delete(
            refresh_tokens::table
                .filter(refresh_tokens::expires_at.lt(now))
                .or_filter(refresh_tokens::revoked_at.is_not_null()),
        )
        .execute(&mut conn)
        .await
        .map_err(|e| AuthRepositoryError::DatabaseError(e.to_string()))?;

        Ok(rows_affected as u64)
    }
}
