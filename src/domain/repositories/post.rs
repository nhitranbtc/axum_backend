use crate::domain::entities::Post;
use async_trait::async_trait;
use uuid::Uuid;

/// Repository trait for Post entity
#[async_trait]
pub trait PostRepository: Send + Sync {
    async fn save(&self, post: &Post) -> Result<Post, PostRepositoryError>;
    async fn update(&self, post: &Post) -> Result<Post, PostRepositoryError>;
    async fn find_by_id(&self, post_id: Uuid) -> Result<Option<Post>, PostRepositoryError>;
    async fn find_by_slug(&self, slug: &str) -> Result<Option<Post>, PostRepositoryError>;
    async fn list_recent(
        &self,
        status: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Post>, PostRepositoryError>;
    async fn list_by_author(
        &self,
        author_id: Uuid,
        status: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Post>, PostRepositoryError>;
    async fn soft_delete(&self, post_id: Uuid) -> Result<bool, PostRepositoryError>;
    async fn count(&self, status: Option<&str>) -> Result<i64, PostRepositoryError>;
}

/// Repository-specific errors for posts
#[derive(Debug, thiserror::Error)]
pub enum PostRepositoryError {
    #[error("Resource already exists: {0}")]
    AlreadyExists(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Resource not found")]
    NotFound,

    #[error("Internal error: {0}")]
    Internal(String),
}
