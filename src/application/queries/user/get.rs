use crate::{
    domain::{
        entities::User, repositories::user_repository::UserRepository, value_objects::UserId,
    },
    shared::AppError,
};
use std::sync::Arc;

/// Query for getting a single user by ID (Read operation - optimized)
pub struct GetUserQuery<R: UserRepository> {
    user_repository: Arc<R>,
}

impl<R: UserRepository> GetUserQuery<R> {
    pub fn new(user_repository: Arc<R>) -> Self {
        Self { user_repository }
    }

    pub async fn execute(&self, user_id: UserId) -> Result<User, AppError> {
        // Future: Add caching layer here
        // if let Some(cached_user) = cache.get(&user_id).await {
        //     return Ok(cached_user);
        // }

        let user = self
            .user_repository
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("User {} not found", user_id)))?;

        // Future: Cache the result
        // cache.set(&user_id, &user, Duration::from_secs(300)).await;

        Ok(user)
    }
}
