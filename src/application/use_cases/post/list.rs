use crate::{
    application::dto::{PostListResponseDto, PostResponseDto},
    domain::{entities::PostStatus, repositories::post::PostRepository},
    shared::AppError,
};
use std::sync::Arc;

pub struct ListPostsUseCase<R: PostRepository> {
    post_repository: Arc<R>,
}

impl<R: PostRepository> ListPostsUseCase<R> {
    pub fn new(post_repository: Arc<R>) -> Self {
        Self { post_repository }
    }

    pub async fn execute(
        &self,
        page: i64,
        page_size: i64,
        status: Option<&str>,
    ) -> Result<PostListResponseDto, AppError> {
        if page < 1 {
            return Err(AppError::Validation("Page must be >= 1".to_string()));
        }
        if !(1..=100).contains(&page_size) {
            return Err(AppError::Validation("Page size must be between 1 and 100".to_string()));
        }

        if let Some(raw_status) = status {
            PostStatus::parse(raw_status)
                .ok_or_else(|| AppError::Validation("Invalid status value".to_string()))?;
        }

        let offset = (page - 1)
            .checked_mul(page_size)
            .ok_or_else(|| AppError::Validation("Pagination overflow".to_string()))?;
        let posts = self.post_repository.list_recent(status, page_size, offset).await?;
        let total_items = self.post_repository.count(status).await?;
        let total_pages =
            if total_items == 0 { 0 } else { (total_items + page_size - 1) / page_size };

        let next_cursor =
            if page < total_pages { Some(format!("page:{}", page + 1)) } else { None };

        let items = posts.iter().map(PostResponseDto::from).collect();

        Ok(PostListResponseDto { items, page, page_size, total_items, total_pages, next_cursor })
    }
}
