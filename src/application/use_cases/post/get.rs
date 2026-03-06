use crate::{
    application::use_cases::post::cache::{cache_ttl, deserialize_post, post_key, serialize_post},
    domain::repositories::post::PostRepository,
    infrastructure::cache::CacheRepository,
    shared::AppError,
};
use std::sync::Arc;
use tracing::{debug, warn};
use uuid::Uuid;

pub struct GetPostUseCase<R: PostRepository, C: CacheRepository + ?Sized> {
    post_repository: Arc<R>,
    cache_repository: Arc<C>,
}

impl<R: PostRepository, C: CacheRepository + ?Sized> GetPostUseCase<R, C> {
    pub fn new(post_repository: Arc<R>, cache_repository: Arc<C>) -> Self {
        Self { post_repository, cache_repository }
    }

    pub async fn execute(&self, post_id: &str) -> Result<crate::domain::entities::Post, AppError> {
        let post_uuid = Uuid::parse_str(post_id)
            .map_err(|_| AppError::Validation("Invalid post ID format".to_string()))?;
        let cache_key = post_key(post_uuid);
        let mut db_query_reason = "cache_miss";

        match self.cache_repository.get(&cache_key).await {
            Ok(Some(cached)) => match deserialize_post(&cached) {
                Ok(post) if !post.is_deleted() => {
                    debug!(post_id = %post_id, cache_key = %cache_key, "post cache hit");
                    return Ok(post);
                },
                Ok(_) => {
                    db_query_reason = "cached_post_deleted";
                    debug!(
                        post_id = %post_id,
                        cache_key = %cache_key,
                        "cached post is soft-deleted, falling back to repository"
                    );
                },
                Err(e) => {
                    db_query_reason = "cache_deserialize_error";
                    warn!(
                        post_id = %post_id,
                        cache_key = %cache_key,
                        error = %e,
                        "failed to deserialize cached post, falling back to repository"
                    );
                },
            },
            Ok(None) => debug!(post_id = %post_id, cache_key = %cache_key, "post cache miss"),
            Err(e) => {
                db_query_reason = "cache_read_error";
                warn!(
                    post_id = %post_id,
                    cache_key = %cache_key,
                    error = %e,
                    "failed to read post from cache, falling back to repository"
                );
            },
        }

        debug!(
            post_id = %post_id,
            cache_key = %cache_key,
            reason = db_query_reason,
            "querying post from database after redis lookup"
        );

        let post = self
            .post_repository
            .find_by_id(post_uuid)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Post with ID {} not found", post_id)))?;

        if post.is_deleted() {
            return Err(AppError::NotFound(format!("Post with ID {} not found", post_id)));
        }

        match serialize_post(&post) {
            Ok(serialized) => {
                if let Err(e) =
                    self.cache_repository.set(&cache_key, &serialized, cache_ttl()).await
                {
                    warn!(
                        post_id = %post_id,
                        cache_key = %cache_key,
                        error = %e,
                        "failed to populate post cache"
                    );
                }
            },
            Err(e) => warn!(
                post_id = %post_id,
                cache_key = %cache_key,
                error = %e,
                "failed to serialize post for cache"
            ),
        }

        Ok(post)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        entities::{Post, PostStatus},
        repositories::post::{PostRepository, PostRepositoryError},
    };
    use async_trait::async_trait;
    use std::sync::{Arc, Mutex};

    struct MockPostRepo {
        post: Option<Post>,
        find_by_id_calls: Arc<Mutex<usize>>,
    }

    #[async_trait]
    impl PostRepository for MockPostRepo {
        async fn save(&self, _post: &Post) -> Result<Post, PostRepositoryError> {
            unreachable!("not used in this test")
        }

        async fn update(&self, _post: &Post) -> Result<Post, PostRepositoryError> {
            unreachable!("not used in this test")
        }

        async fn find_by_id(&self, _post_id: Uuid) -> Result<Option<Post>, PostRepositoryError> {
            *self.find_by_id_calls.lock().expect("poisoned lock") += 1;
            Ok(self.post.clone())
        }

        async fn find_by_slug(&self, _slug: &str) -> Result<Option<Post>, PostRepositoryError> {
            unreachable!("not used in this test")
        }

        async fn list_recent(
            &self,
            _status: Option<&str>,
            _limit: i64,
            _offset: i64,
        ) -> Result<Vec<Post>, PostRepositoryError> {
            unreachable!("not used in this test")
        }

        async fn list_by_author(
            &self,
            _author_id: Uuid,
            _status: Option<&str>,
            _limit: i64,
            _offset: i64,
        ) -> Result<Vec<Post>, PostRepositoryError> {
            unreachable!("not used in this test")
        }

        async fn soft_delete(&self, _post_id: Uuid) -> Result<bool, PostRepositoryError> {
            unreachable!("not used in this test")
        }

        async fn count(&self, _status: Option<&str>) -> Result<i64, PostRepositoryError> {
            unreachable!("not used in this test")
        }
    }

    struct MockCacheRepo {
        get_value: Option<String>,
        should_fail_get: bool,
        set_calls: Arc<Mutex<usize>>,
    }

    #[async_trait]
    impl CacheRepository for MockCacheRepo {
        async fn get(
            &self,
            _key: &str,
        ) -> Result<Option<String>, crate::infrastructure::cache::CacheError> {
            if self.should_fail_get {
                return Err(crate::infrastructure::cache::CacheError::Connection(
                    "cache unavailable".to_string(),
                ));
            }
            Ok(self.get_value.clone())
        }

        async fn set(
            &self,
            _key: &str,
            _value: &str,
            _ttl: std::time::Duration,
        ) -> Result<(), crate::infrastructure::cache::CacheError> {
            *self.set_calls.lock().expect("poisoned lock") += 1;
            Ok(())
        }

        async fn delete(&self, _key: &str) -> Result<(), crate::infrastructure::cache::CacheError> {
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

    fn sample_post(id: Uuid) -> Post {
        Post::from_existing(
            id,
            Uuid::new_v4(),
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
    async fn execute_returns_from_cache_on_hit() {
        let post_id = Uuid::new_v4();
        let post = sample_post(post_id);
        let cached = serde_json::to_string(&post).expect("serialize post");
        let repo_calls = Arc::new(Mutex::new(0usize));
        let set_calls = Arc::new(Mutex::new(0usize));

        let use_case = GetPostUseCase::new(
            Arc::new(MockPostRepo {
                post: Some(post.clone()),
                find_by_id_calls: repo_calls.clone(),
            }),
            Arc::new(MockCacheRepo {
                get_value: Some(cached),
                should_fail_get: false,
                set_calls: set_calls.clone(),
            }),
        );

        let result =
            use_case.execute(&post_id.to_string()).await.expect("cache hit should succeed");
        assert_eq!(result.id, post.id);
        assert_eq!(*repo_calls.lock().expect("poisoned lock"), 0);
        assert_eq!(*set_calls.lock().expect("poisoned lock"), 0);
    }

    #[tokio::test]
    async fn execute_falls_back_to_repo_when_cache_get_fails() {
        let post_id = Uuid::new_v4();
        let post = sample_post(post_id);
        let repo_calls = Arc::new(Mutex::new(0usize));
        let set_calls = Arc::new(Mutex::new(0usize));

        let use_case = GetPostUseCase::new(
            Arc::new(MockPostRepo {
                post: Some(post.clone()),
                find_by_id_calls: repo_calls.clone(),
            }),
            Arc::new(MockCacheRepo {
                get_value: None,
                should_fail_get: true,
                set_calls: set_calls.clone(),
            }),
        );

        let result = use_case.execute(&post_id.to_string()).await.expect("fallback should succeed");
        assert_eq!(result.id, post.id);
        assert_eq!(*repo_calls.lock().expect("poisoned lock"), 1);
        assert_eq!(*set_calls.lock().expect("poisoned lock"), 1);
    }
}
