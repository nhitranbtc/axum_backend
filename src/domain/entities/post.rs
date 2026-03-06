use crate::domain::errors::DomainError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PostStatus {
    Draft,
    Published,
    Archived,
}

impl Default for PostStatus {
    fn default() -> Self {
        Self::Draft
    }
}

impl PostStatus {
    pub fn parse(input: &str) -> Option<Self> {
        match input.to_lowercase().as_str() {
            "draft" => Some(Self::Draft),
            "published" => Some(Self::Published),
            "archived" => Some(Self::Archived),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Published => "published",
            Self::Archived => "archived",
        }
    }
}

/// Post domain entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: Uuid,
    pub author_id: Uuid,
    pub title: String,
    pub slug: String,
    pub content: String,
    pub status: PostStatus,
    pub tags: Vec<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Post {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        author_id: Uuid,
        title: String,
        slug: String,
        content: String,
        status: PostStatus,
        tags: Vec<String>,
    ) -> Result<Self, DomainError> {
        let title = title.trim();
        let slug = slug.trim();
        let content = content.trim();

        if title.is_empty() {
            return Err(DomainError::ValidationError("Title cannot be empty".to_string()));
        }
        if title.len() > 255 {
            return Err(DomainError::ValidationError(
                "Title must be 255 characters or less".to_string(),
            ));
        }
        if slug.is_empty() {
            return Err(DomainError::ValidationError("Slug cannot be empty".to_string()));
        }
        if content.is_empty() {
            return Err(DomainError::ValidationError("Content cannot be empty".to_string()));
        }

        if tags.iter().any(|tag| tag.trim().is_empty()) {
            return Err(DomainError::ValidationError(
                "Tags cannot contain empty values".to_string(),
            ));
        }

        let now = Utc::now();
        let published_at = if status == PostStatus::Published { Some(now) } else { None };

        Ok(Self {
            id: Uuid::new_v4(),
            author_id,
            title: title.to_string(),
            slug: slug.to_string(),
            content: content.to_string(),
            status,
            tags,
            published_at,
            deleted_at: None,
            created_at: now,
            updated_at: now,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn from_existing(
        id: Uuid,
        author_id: Uuid,
        title: String,
        slug: String,
        content: String,
        status: PostStatus,
        tags: Vec<String>,
        published_at: Option<DateTime<Utc>>,
        deleted_at: Option<DateTime<Utc>>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            author_id,
            title,
            slug,
            content,
            status,
            tags,
            published_at,
            deleted_at,
            created_at,
            updated_at,
        }
    }

    pub fn update_title_and_slug(
        &mut self,
        title: String,
        slug: String,
    ) -> Result<(), DomainError> {
        let title = title.trim();
        let slug = slug.trim();

        if title.is_empty() {
            return Err(DomainError::ValidationError("Title cannot be empty".to_string()));
        }
        if title.len() > 255 {
            return Err(DomainError::ValidationError(
                "Title must be 255 characters or less".to_string(),
            ));
        }
        if slug.is_empty() {
            return Err(DomainError::ValidationError("Slug cannot be empty".to_string()));
        }

        self.title = title.to_string();
        self.slug = slug.to_string();
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn update_content(&mut self, content: String) -> Result<(), DomainError> {
        let content = content.trim();
        if content.is_empty() {
            return Err(DomainError::ValidationError("Content cannot be empty".to_string()));
        }

        self.content = content.to_string();
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn update_tags(&mut self, tags: Vec<String>) -> Result<(), DomainError> {
        if tags.iter().any(|tag| tag.trim().is_empty()) {
            return Err(DomainError::ValidationError(
                "Tags cannot contain empty values".to_string(),
            ));
        }
        self.tags = tags;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn update_status(&mut self, status: PostStatus) -> Result<(), DomainError> {
        let old = self.status;
        let valid = match (old, status) {
            (PostStatus::Draft, PostStatus::Published | PostStatus::Archived)
            | (PostStatus::Published, PostStatus::Archived)
            | (PostStatus::Archived, PostStatus::Draft)
            | (PostStatus::Draft, PostStatus::Draft)
            | (PostStatus::Published, PostStatus::Published)
            | (PostStatus::Archived, PostStatus::Archived) => true,
            _ => false,
        };

        if !valid {
            return Err(DomainError::ValidationError(format!(
                "Invalid status transition: {} -> {}",
                old.as_str(),
                status.as_str()
            )));
        }

        self.status = status;
        if status == PostStatus::Published && self.published_at.is_none() {
            self.published_at = Some(Utc::now());
        }
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn soft_delete(&mut self) {
        let now = Utc::now();
        self.deleted_at = Some(now);
        self.updated_at = now;
    }

    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }
}
