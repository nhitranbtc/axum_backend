use crate::{
    application::dto::CreatePostDto,
    application::use_cases::post::helpers::{generate_unique_slug, parse_status},
    domain::{
        entities::Post,
        repositories::post::{PostRepository, PostRepositoryError},
    },
    shared::AppError,
};
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

/// Use case for creating a new post
pub struct CreatePostUseCase<R: PostRepository> {
    post_repository: Arc<R>,
}

impl<R: PostRepository> CreatePostUseCase<R> {
    pub fn new(post_repository: Arc<R>) -> Self {
        Self { post_repository }
    }

    pub async fn execute(&self, author_id: Uuid, dto: CreatePostDto) -> Result<Post, AppError> {
        dto.validate().map_err(|e| AppError::Validation(e.to_string()))?;

        let status = parse_status(dto.status.as_deref())?;

        let slug = generate_unique_slug(self.post_repository.as_ref(), &dto.title, None).await?;

        let post = Post::new(author_id, dto.title, slug, dto.content, status, dto.tags)
            .map_err(|e| AppError::Validation(e.to_string()))?;

        let saved_post = self.post_repository.save(&post).await.map_err(|e| match e {
            PostRepositoryError::AlreadyExists(msg) => AppError::Validation(msg),
            PostRepositoryError::Conflict(msg) => AppError::Validation(msg),
            PostRepositoryError::NotFound => AppError::NotFound("Post not found".to_string()),
            PostRepositoryError::Internal(msg) => {
                AppError::Internal(anyhow::anyhow!("Post repository error: {}", msg))
            },
        })?;

        tracing::info!("Post created successfully: {}", saved_post.id);

        Ok(saved_post)
    }
}
