use crate::{
    application::dto::UpdatePostDto,
    application::use_cases::post::cache::{post_key, post_slug_key},
    application::use_cases::post::helpers::{ensure_actor_can_manage_post, generate_unique_slug},
    domain::{
        entities::{Post, PostStatus},
        repositories::{post::PostRepository, user_repository::UserRepository},
    },
    infrastructure::cache::CacheRepository,
    shared::AppError,
};
use std::sync::Arc;
use tracing::warn;
use uuid::Uuid;
use validator::Validate;

pub struct UpdatePostUseCase<PR: PostRepository, UR: UserRepository, C: CacheRepository + ?Sized> {
    post_repository: Arc<PR>,
    user_repository: Arc<UR>,
    cache_repository: Arc<C>,
}

impl<PR: PostRepository, UR: UserRepository, C: CacheRepository + ?Sized>
    UpdatePostUseCase<PR, UR, C>
{
    pub fn new(
        post_repository: Arc<PR>,
        user_repository: Arc<UR>,
        cache_repository: Arc<C>,
    ) -> Self {
        Self { post_repository, user_repository, cache_repository }
    }

    pub async fn execute(
        &self,
        actor_id: Uuid,
        post_id: &str,
        dto: UpdatePostDto,
    ) -> Result<Post, AppError> {
        dto.validate().map_err(|e| AppError::Validation(e.to_string()))?;

        let post_uuid = Uuid::parse_str(post_id)
            .map_err(|_| AppError::Validation("Invalid post ID format".to_string()))?;

        let mut post = self
            .post_repository
            .find_by_id(post_uuid)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Post with ID {} not found", post_id)))?;

        if post.is_deleted() {
            return Err(AppError::NotFound(format!("Post with ID {} not found", post_id)));
        }

        let old_slug = post.slug.clone();

        ensure_actor_can_manage_post(self.user_repository.as_ref(), actor_id, post.author_id)
            .await?;

        if let Some(title) = dto.title {
            let slug =
                generate_unique_slug(self.post_repository.as_ref(), &title, Some(post.id)).await?;
            post.update_title_and_slug(title, slug)
                .map_err(|e| AppError::Validation(e.to_string()))?;
        }

        if let Some(content) = dto.content {
            post.update_content(content).map_err(|e| AppError::Validation(e.to_string()))?;
        }

        if let Some(status) = dto.status {
            let status = PostStatus::parse(&status)
                .ok_or_else(|| AppError::Validation("Invalid status value".to_string()))?;
            post.update_status(status).map_err(|e| AppError::Validation(e.to_string()))?;
        }

        if let Some(tags) = dto.tags {
            post.update_tags(tags).map_err(|e| AppError::Validation(e.to_string()))?;
        }

        let updated_post = self.post_repository.update(&post).await.map_err(map_repo_err)?;
        self.invalidate_post_cache(updated_post.id, &old_slug, &updated_post.slug).await;

        Ok(updated_post)
    }

    async fn invalidate_post_cache(&self, post_id: Uuid, old_slug: &str, new_slug: &str) {
        let id_key = post_key(post_id);
        if let Err(e) = self.cache_repository.delete(&id_key).await {
            warn!(post_id = %post_id, cache_key = %id_key, error = %e, "failed to invalidate post id cache");
        }

        let old_slug_key = post_slug_key(old_slug);
        if let Err(e) = self.cache_repository.delete(&old_slug_key).await {
            warn!(
                post_id = %post_id,
                cache_key = %old_slug_key,
                error = %e,
                "failed to invalidate old post slug cache"
            );
        }

        let new_slug_key = post_slug_key(new_slug);
        if old_slug_key != new_slug_key {
            if let Err(e) = self.cache_repository.delete(&new_slug_key).await {
                warn!(
                    post_id = %post_id,
                    cache_key = %new_slug_key,
                    error = %e,
                    "failed to invalidate new post slug cache"
                );
            }
        }
    }
}

fn map_repo_err(err: crate::domain::repositories::post::PostRepositoryError) -> AppError {
    match err {
        crate::domain::repositories::post::PostRepositoryError::AlreadyExists(msg) => {
            AppError::Validation(msg)
        },
        crate::domain::repositories::post::PostRepositoryError::Conflict(msg) => {
            AppError::Validation(msg)
        },
        crate::domain::repositories::post::PostRepositoryError::NotFound => {
            AppError::NotFound("Post not found".to_string())
        },
        crate::domain::repositories::post::PostRepositoryError::Internal(msg) => {
            AppError::Internal(anyhow::anyhow!("Post repository error: {}", msg))
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        entities::User,
        repositories::{
            post::{PostRepository, PostRepositoryError},
            user_repository::{RepositoryError, UserRepository},
        },
        value_objects::{Email, UserId, UserRole},
    };
    use async_trait::async_trait;
    use std::sync::{Arc, Mutex};

    struct MockPostRepo {
        post: Post,
    }

    #[async_trait]
    impl PostRepository for MockPostRepo {
        async fn save(&self, _post: &Post) -> Result<Post, PostRepositoryError> {
            unreachable!("not used")
        }

        async fn update(&self, post: &Post) -> Result<Post, PostRepositoryError> {
            Ok(post.clone())
        }

        async fn find_by_id(&self, _post_id: Uuid) -> Result<Option<Post>, PostRepositoryError> {
            Ok(Some(self.post.clone()))
        }

        async fn find_by_slug(&self, _slug: &str) -> Result<Option<Post>, PostRepositoryError> {
            Ok(None)
        }

        async fn list_recent(
            &self,
            _status: Option<&str>,
            _limit: i64,
            _offset: i64,
        ) -> Result<Vec<Post>, PostRepositoryError> {
            unreachable!("not used")
        }

        async fn list_by_author(
            &self,
            _author_id: Uuid,
            _status: Option<&str>,
            _limit: i64,
            _offset: i64,
        ) -> Result<Vec<Post>, PostRepositoryError> {
            unreachable!("not used")
        }

        async fn soft_delete(&self, _post_id: Uuid) -> Result<bool, PostRepositoryError> {
            unreachable!("not used")
        }

        async fn count(&self, _status: Option<&str>) -> Result<i64, PostRepositoryError> {
            unreachable!("not used")
        }
    }

    struct MockUserRepo {
        user: User,
    }

    #[async_trait]
    impl UserRepository for MockUserRepo {
        async fn save(&self, _user: &User) -> Result<User, RepositoryError> {
            unreachable!("not used")
        }

        async fn update(&self, _user: &User) -> Result<User, RepositoryError> {
            unreachable!("not used")
        }

        async fn find_by_id(&self, _id: UserId) -> Result<Option<User>, RepositoryError> {
            Ok(Some(self.user.clone()))
        }

        async fn find_by_email(&self, _email: &Email) -> Result<Option<User>, RepositoryError> {
            unreachable!("not used")
        }

        async fn exists_by_email(&self, _email: &Email) -> Result<bool, RepositoryError> {
            unreachable!("not used")
        }

        async fn count(&self) -> Result<i64, RepositoryError> {
            unreachable!("not used")
        }

        async fn list_paginated(
            &self,
            _limit: i64,
            _offset: i64,
        ) -> Result<Vec<User>, RepositoryError> {
            unreachable!("not used")
        }

        async fn delete(&self, _id: UserId) -> Result<bool, RepositoryError> {
            unreachable!("not used")
        }

        async fn delete_all(&self) -> Result<usize, RepositoryError> {
            unreachable!("not used")
        }
    }

    struct MockCacheRepo {
        deleted_keys: Arc<Mutex<Vec<String>>>,
    }

    #[async_trait]
    impl CacheRepository for MockCacheRepo {
        async fn get(
            &self,
            _key: &str,
        ) -> Result<Option<String>, crate::infrastructure::cache::CacheError> {
            Ok(None)
        }

        async fn set(
            &self,
            _key: &str,
            _value: &str,
            _ttl: std::time::Duration,
        ) -> Result<(), crate::infrastructure::cache::CacheError> {
            Ok(())
        }

        async fn delete(&self, key: &str) -> Result<(), crate::infrastructure::cache::CacheError> {
            self.deleted_keys.lock().expect("poisoned lock").push(key.to_string());
            Ok(())
        }

        async fn set_nx(
            &self,
            _key: &str,
            _value: &str,
            _ttl: std::time::Duration,
        ) -> Result<bool, crate::infrastructure::cache::CacheError> {
            Ok(true)
        }

        async fn delete_if_equals(
            &self,
            _key: &str,
            _value: &str,
        ) -> Result<bool, crate::infrastructure::cache::CacheError> {
            Ok(true)
        }
    }

    fn sample_user(actor_id: Uuid) -> User {
        User::from_existing(
            UserId::from_uuid(actor_id),
            Email::parse("post-cache-update@test.com").expect("valid email"),
            "actor".to_string(),
            None,
            UserRole::Viewer,
            true,
            true,
            None,
            None,
            None,
            chrono::Utc::now(),
            chrono::Utc::now(),
        )
    }

    fn sample_post(post_id: Uuid, author_id: Uuid) -> Post {
        Post::from_existing(
            post_id,
            author_id,
            "Title".to_string(),
            "title".to_string(),
            "Content".to_string(),
            PostStatus::Draft,
            vec![],
            None,
            None,
            chrono::Utc::now(),
            chrono::Utc::now(),
        )
    }

    #[tokio::test]
    async fn update_invalidates_id_and_slug_cache_keys() {
        let actor_id = Uuid::new_v4();
        let post_id = Uuid::new_v4();
        let deleted_keys = Arc::new(Mutex::new(Vec::<String>::new()));
        let use_case = UpdatePostUseCase::new(
            Arc::new(MockPostRepo { post: sample_post(post_id, actor_id) }),
            Arc::new(MockUserRepo { user: sample_user(actor_id) }),
            Arc::new(MockCacheRepo { deleted_keys: deleted_keys.clone() }),
        );

        let updated = use_case
            .execute(
                actor_id,
                &post_id.to_string(),
                UpdatePostDto {
                    title: None,
                    content: Some("Updated".to_string()),
                    status: None,
                    tags: None,
                },
            )
            .await
            .expect("update should succeed");
        assert_eq!(updated.id, post_id);

        let keys = deleted_keys.lock().expect("poisoned lock").clone();
        assert!(keys.contains(&format!("post:{post_id}")));
        assert!(keys.contains(&"post:slug:title".to_string()));
    }
}
