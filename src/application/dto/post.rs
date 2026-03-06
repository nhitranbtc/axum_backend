use crate::domain::entities::Post;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

/// DTO for creating a new post
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[schema(example = json!({
    "title": "My first post",
    "content": "This is the content of my first post."
}))]
pub struct CreatePostDto {
    #[validate(length(min = 1, max = 255))]
    pub title: String,

    #[validate(length(min = 1, max = 10000))]
    pub content: String,

    pub status: Option<String>,

    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[schema(example = json!({
    "title": "Updated post title",
    "content": "Updated content",
    "status": "published",
    "tags": ["rust", "backend"]
}))]
pub struct UpdatePostDto {
    #[validate(length(min = 1, max = 255))]
    pub title: Option<String>,

    #[validate(length(min = 1, max = 10000))]
    pub content: Option<String>,

    pub status: Option<String>,

    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ListPostsQueryDto {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_page_size", alias = "pageSize")]
    pub page_size: i64,
    pub status: Option<String>,
}

fn default_page() -> i64 {
    1
}

fn default_page_size() -> i64 {
    10
}

/// DTO for post response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PostResponseDto {
    pub id: String,
    pub author_id: String,
    pub title: String,
    pub slug: String,
    pub content: String,
    pub status: String,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Post> for PostResponseDto {
    fn from(post: Post) -> Self {
        Self {
            id: post.id.to_string(),
            author_id: post.author_id.to_string(),
            title: post.title,
            slug: post.slug,
            content: post.content,
            status: post.status.as_str().to_string(),
            tags: post.tags,
            created_at: post.created_at.to_rfc3339(),
            updated_at: post.updated_at.to_rfc3339(),
        }
    }
}

impl From<&Post> for PostResponseDto {
    fn from(post: &Post) -> Self {
        Self {
            id: post.id.to_string(),
            author_id: post.author_id.to_string(),
            title: post.title.clone(),
            slug: post.slug.clone(),
            content: post.content.clone(),
            status: post.status.as_str().to_string(),
            tags: post.tags.clone(),
            created_at: post.created_at.to_rfc3339(),
            updated_at: post.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PostListResponseDto {
    pub items: Vec<PostResponseDto>,
    pub page: i64,
    pub page_size: i64,
    pub total_items: i64,
    pub total_pages: i64,
    pub next_cursor: Option<String>,
}
