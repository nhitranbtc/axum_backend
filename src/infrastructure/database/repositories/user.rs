use crate::{
    domain::{
        entities::User,
        repositories::user_repository::{RepositoryError, UserRepository},
        value_objects::{Email, UserId, UserRole},
    },
    infrastructure::database::{models::UserModel, schema::users, DbPool},
};
use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

/// PostgreSQL implementation of UserRepository
///
/// This implementation uses Diesel ORM with async PostgreSQL.
/// The struct name is generic to avoid coupling to specific database technology.
#[derive(Clone)]
pub struct RepositoryImpl {
    pool: DbPool,
}

impl RepositoryImpl {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// Helper: Convert UserModel to domain User entity
    fn model_to_entity(model: UserModel) -> Result<User, RepositoryError> {
        Ok(User::from_existing(
            UserId::from_uuid(model.id),
            Email::parse(&model.email).map_err(|e| {
                RepositoryError::Internal(format!("Invalid email from database: {}", e))
            })?,
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
        ))
    }

    /// Helper: Convert domain User entity to UserModel
    fn entity_to_model(user: &User) -> UserModel {
        UserModel {
            id: *user.id.as_uuid(),
            email: user.email.as_str().to_string(),
            name: user.name.clone(),
            created_at: user.created_at,
            updated_at: user.updated_at,
            password_hash: user.password_hash.clone(),
            role: user.role.to_string(),
            is_active: user.is_active,
            last_login: user.last_login,
            confirmation_code: user.confirmation_code.clone(),
            confirmation_code_expires_at: user.confirmation_code_expires_at,
            email_verified: user.is_email_verified,
        }
    }
}

#[async_trait]
impl UserRepository for RepositoryImpl {
    async fn save(&self, user: &User) -> Result<User, RepositoryError> {
        let mut conn =
            self.pool.get().await.map_err(|e| {
                RepositoryError::Internal(format!("Failed to get connection: {}", e))
            })?;

        let db_user = Self::entity_to_model(user);

        let result = diesel::insert_into(users::table)
            .values(&db_user)
            .on_conflict(users::id)
            .do_update()
            .set(&db_user)
            .get_result::<UserModel>(&mut conn)
            .await?;

        Self::model_to_entity(result)
    }

    async fn update(&self, user: &User) -> Result<User, RepositoryError> {
        let mut conn =
            self.pool.get().await.map_err(|e| {
                RepositoryError::Internal(format!("Failed to get connection: {}", e))
            })?;

        let db_user = Self::entity_to_model(user);

        let result = diesel::update(users::table.filter(users::id.eq(user.id.as_uuid())))
            .set(&db_user)
            .get_result::<UserModel>(&mut conn)
            .await?;

        Self::model_to_entity(result)
    }

    async fn find_by_id(&self, id: UserId) -> Result<Option<User>, RepositoryError> {
        let mut conn =
            self.pool.get().await.map_err(|e| {
                RepositoryError::Internal(format!("Failed to get connection: {}", e))
            })?;

        let result = users::table
            .filter(users::id.eq(id.as_uuid()))
            .first::<UserModel>(&mut conn)
            .await
            .optional()?;

        result.map(Self::model_to_entity).transpose()
    }

    async fn find_by_email(&self, email: &Email) -> Result<Option<User>, RepositoryError> {
        let mut conn =
            self.pool.get().await.map_err(|e| {
                RepositoryError::Internal(format!("Failed to get connection: {}", e))
            })?;

        let result = users::table
            .filter(users::email.eq(email.as_str()))
            .first::<UserModel>(&mut conn)
            .await
            .optional()?;

        result.map(Self::model_to_entity).transpose()
    }

    async fn exists_by_email(&self, email: &Email) -> Result<bool, RepositoryError> {
        let mut conn =
            self.pool.get().await.map_err(|e| {
                RepositoryError::Internal(format!("Failed to get connection: {}", e))
            })?;

        let count: i64 = users::table
            .filter(users::email.eq(email.as_str()))
            .count()
            .get_result(&mut conn)
            .await?;

        Ok(count > 0)
    }

    async fn count(&self) -> Result<i64, RepositoryError> {
        let mut conn =
            self.pool.get().await.map_err(|e| {
                RepositoryError::Internal(format!("Failed to get connection: {}", e))
            })?;

        let count: i64 = users::table.count().get_result(&mut conn).await?;

        Ok(count)
    }

    async fn list_paginated(&self, limit: i64, offset: i64) -> Result<Vec<User>, RepositoryError> {
        let mut conn =
            self.pool.get().await.map_err(|e| {
                RepositoryError::Internal(format!("Failed to get connection: {}", e))
            })?;

        let results = users::table
            .order(users::created_at.desc())
            .limit(limit)
            .offset(offset)
            .load::<UserModel>(&mut conn)
            .await?;

        results.into_iter().map(Self::model_to_entity).collect::<Result<Vec<_>, _>>()
    }

    async fn delete(&self, id: UserId) -> Result<bool, RepositoryError> {
        let mut conn =
            self.pool.get().await.map_err(|e| {
                RepositoryError::Internal(format!("Failed to get connection: {}", e))
            })?;

        let rows_affected = diesel::delete(users::table.filter(users::id.eq(id.as_uuid())))
            .execute(&mut conn)
            .await?;

        Ok(rows_affected > 0)
    }

    async fn delete_all(&self) -> Result<usize, RepositoryError> {
        let mut conn =
            self.pool.get().await.map_err(|e| {
                RepositoryError::Internal(format!("Failed to get connection: {}", e))
            })?;

        let rows_affected = diesel::delete(users::table).execute(&mut conn).await?;

        Ok(rows_affected)
    }
}
