use async_trait::async_trait;
use scylla::client::caching_session::CachingSession;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::{
    entities::{Post, PostStatus},
    repositories::post::{PostRepository, PostRepositoryError},
};
use crate::infrastructure::database::scylla::{
    connection::ScyllaSession, models::PostRow, operations::prelude::*,
};

#[derive(Clone)]
pub struct RepositoryImpl {
    session: Arc<CachingSession>,
}

impl RepositoryImpl {
    pub fn new(session: Arc<ScyllaSession>) -> Self {
        Self { session: session.session() }
    }

    fn db_err(e: impl std::fmt::Display) -> PostRepositoryError {
        PostRepositoryError::Internal(e.to_string())
    }

    fn row_to_entity(row: PostRow) -> Post {
        Post::from_existing(
            row.post_id,
            row.author_id,
            row.title,
            row.slug,
            row.content,
            PostStatus::parse(&row.status).unwrap_or_default(),
            row.tags,
            PostRow::from_opt_ts(row.published_at),
            PostRow::from_opt_ts(row.deleted_at),
            PostRow::from_ts(row.created_at),
            PostRow::from_ts(row.updated_at),
        )
    }
}

#[async_trait]
impl PostRepository for RepositoryImpl {
    async fn save(&self, post: &Post) -> Result<Post, PostRepositoryError> {
        let row = PostRow {
            post_id: post.id,
            author_id: post.author_id,
            title: post.title.clone(),
            slug: post.slug.clone(),
            content: post.content.clone(),
            status: post.status.as_str().to_string(),
            tags: post.tags.clone(),
            published_at: PostRow::opt_ts(post.published_at),
            deleted_at: PostRow::opt_ts(post.deleted_at),
            created_at: PostRow::ts(post.created_at),
            updated_at: PostRow::ts(post.updated_at),
        };

        row.insert().execute(&self.session).await.map_err(Self::db_err)?;

        Ok(Self::row_to_entity(row))
    }

    async fn update(&self, post: &Post) -> Result<Post, PostRepositoryError> {
        execute_unpaged(
            &self.session,
            PostRow::UPDATE_QUERY,
            (
                post.title.clone(),
                post.slug.clone(),
                post.content.clone(),
                post.status.as_str().to_string(),
                post.tags.clone(),
                PostRow::opt_ts(post.published_at),
                PostRow::ts(post.updated_at),
                post.id,
            ),
        )
        .await
        .map_err(Self::db_err)?;

        Ok(post.clone())
    }

    async fn find_by_id(&self, post_id: Uuid) -> Result<Option<Post>, PostRepositoryError> {
        let row = PostRow::maybe_find_by_primary_key_value((post_id,))
            .execute(&self.session)
            .await
            .map_err(Self::db_err)?;
        Ok(row.map(Self::row_to_entity))
    }

    async fn find_by_slug(&self, slug: &str) -> Result<Option<Post>, PostRepositoryError> {
        let row = PostRow::maybe_find_first(PostRow::FIND_BY_SLUG_QUERY, (slug,))
            .execute(&self.session)
            .await
            .map_err(Self::db_err)?;
        Ok(row.map(Self::row_to_entity))
    }

    async fn list_recent(
        &self,
        status: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Post>, PostRepositoryError> {
        let fetch_limit = (limit + offset).max(limit).max(1) as i32;

        let mut posts: Vec<Post> = match status {
            Some(status) => PostRow::find(PostRow::FIND_BY_STATUS_QUERY, (status, fetch_limit))
                .execute(&self.session)
                .await
                .map_err(Self::db_err)?
                .into_iter()
                .map(Self::row_to_entity)
                .collect(),
            None => PostRow::find(PostRow::FIND_ALL_QUERY, (fetch_limit,))
                .execute(&self.session)
                .await
                .map_err(Self::db_err)?
                .into_iter()
                .map(Self::row_to_entity)
                .collect(),
        };

        posts.retain(|p| p.deleted_at.is_none());
        posts.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(posts
            .into_iter()
            .skip(offset.max(0) as usize)
            .take(limit.max(0) as usize)
            .collect())
    }

    async fn list_by_author(
        &self,
        author_id: Uuid,
        status: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Post>, PostRepositoryError> {
        let fetch_limit = (limit + offset).max(limit).max(1) as i32;
        let mut posts = match status {
            Some(status) => PostRow::find(
                PostRow::FIND_BY_AUTHOR_AND_STATUS_QUERY,
                (author_id, status, fetch_limit),
            )
            .execute(&self.session)
            .await
            .map_err(Self::db_err)?
            .into_iter()
            .map(Self::row_to_entity)
            .collect::<Vec<_>>(),
            None => PostRow::find(PostRow::FIND_BY_AUTHOR_QUERY, (author_id, fetch_limit))
                .execute(&self.session)
                .await
                .map_err(Self::db_err)?
                .into_iter()
                .map(Self::row_to_entity)
                .collect::<Vec<_>>(),
        };

        posts.retain(|p| p.deleted_at.is_none());
        posts.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(posts
            .into_iter()
            .skip(offset.max(0) as usize)
            .take(limit.max(0) as usize)
            .collect())
    }

    async fn soft_delete(&self, post_id: Uuid) -> Result<bool, PostRepositoryError> {
        let deleted_at = PostRow::ts(chrono::Utc::now());
        let updated_at = PostRow::ts(chrono::Utc::now());

        if self.find_by_id(post_id).await?.is_none() {
            return Ok(false);
        }

        execute_unpaged(
            &self.session,
            PostRow::SOFT_DELETE_QUERY,
            (deleted_at, updated_at, post_id),
        )
        .await
        .map_err(Self::db_err)?;

        Ok(true)
    }

    async fn count(&self, status: Option<&str>) -> Result<i64, PostRepositoryError> {
        const COUNT_SCAN_LIMIT: i32 = 10_000;
        let rows = match status {
            Some(status) => {
                PostRow::find(PostRow::FIND_BY_STATUS_QUERY, (status, COUNT_SCAN_LIMIT))
                    .execute(&self.session)
                    .await
                    .map_err(Self::db_err)?
            },
            None => PostRow::find(PostRow::FIND_ALL_QUERY, (COUNT_SCAN_LIMIT,))
                .execute(&self.session)
                .await
                .map_err(Self::db_err)?,
        };

        Ok(rows.into_iter().filter(|r| r.deleted_at.is_none()).count() as i64)
    }
}
