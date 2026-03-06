use chrono::{DateTime, Utc};
use scylla::value::CqlTimestamp;
use scylla_macros::{DeserializeRow, SerializeRow};
use uuid::Uuid;

use crate::infrastructure::database::scylla::operations::model::{BaseModel, Model};

#[derive(Debug, Clone, SerializeRow, DeserializeRow)]
pub struct PostRow {
    pub post_id: Uuid,
    pub author_id: Uuid,
    pub title: String,
    pub slug: String,
    pub content: String,
    pub status: String,
    pub tags: Vec<String>,
    pub published_at: Option<CqlTimestamp>,
    pub deleted_at: Option<CqlTimestamp>,
    pub created_at: CqlTimestamp,
    pub updated_at: CqlTimestamp,
}

impl PostRow {
    pub const INSERT_QUERY: &'static str = "INSERT INTO posts \
        (post_id, author_id, title, slug, content, status, tags, published_at, deleted_at, created_at, updated_at) \
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";

    pub const FIND_BY_PRIMARY_KEY_QUERY: &'static str = "SELECT \
        post_id, author_id, title, slug, content, status, tags, published_at, deleted_at, created_at, updated_at \
        FROM posts WHERE post_id = ?";

    pub const FIND_BY_SLUG_QUERY: &'static str = "SELECT \
        post_id, author_id, title, slug, content, status, tags, published_at, deleted_at, created_at, updated_at \
        FROM posts WHERE slug = ? ALLOW FILTERING";

    pub const FIND_ALL_QUERY: &'static str = "SELECT \
        post_id, author_id, title, slug, content, status, tags, published_at, deleted_at, created_at, updated_at \
        FROM posts LIMIT ?";

    pub const FIND_BY_STATUS_QUERY: &'static str = "SELECT \
        post_id, author_id, title, slug, content, status, tags, published_at, deleted_at, created_at, updated_at \
        FROM posts WHERE status = ? LIMIT ? ALLOW FILTERING";

    pub const FIND_BY_AUTHOR_QUERY: &'static str = "SELECT \
        post_id, author_id, title, slug, content, status, tags, published_at, deleted_at, created_at, updated_at \
        FROM posts WHERE author_id = ? LIMIT ? ALLOW FILTERING";

    pub const FIND_BY_AUTHOR_AND_STATUS_QUERY: &'static str = "SELECT \
        post_id, author_id, title, slug, content, status, tags, published_at, deleted_at, created_at, updated_at \
        FROM posts WHERE author_id = ? AND status = ? LIMIT ? ALLOW FILTERING";

    pub const UPDATE_QUERY: &'static str =
        "UPDATE posts SET title = ?, slug = ?, content = ?, status = ?, tags = ?, published_at = ?, updated_at = ? WHERE post_id = ?";

    pub const SOFT_DELETE_QUERY: &'static str =
        "UPDATE posts SET deleted_at = ?, updated_at = ? WHERE post_id = ?";

    pub const DELETE_QUERY: &'static str = "DELETE FROM posts WHERE post_id = ?";

    pub const COUNT_ALL_QUERY: &'static str =
        "SELECT COUNT(*) FROM posts WHERE deleted_at = null ALLOW FILTERING";

    pub const COUNT_BY_STATUS_QUERY: &'static str =
        "SELECT COUNT(*) FROM posts WHERE status = ? AND deleted_at = null ALLOW FILTERING";

    pub fn ts(dt: DateTime<Utc>) -> CqlTimestamp {
        CqlTimestamp(dt.timestamp_millis())
    }

    pub fn opt_ts(dt: Option<DateTime<Utc>>) -> Option<CqlTimestamp> {
        dt.map(Self::ts)
    }

    pub fn from_ts(ts: CqlTimestamp) -> DateTime<Utc> {
        let secs = ts.0 / 1_000;
        let nanos = ((ts.0 % 1_000) * 1_000_000) as u32;
        DateTime::from_timestamp(secs, nanos).unwrap_or_else(Utc::now)
    }

    pub fn from_opt_ts(ts: Option<CqlTimestamp>) -> Option<DateTime<Utc>> {
        ts.map(Self::from_ts)
    }
}

impl BaseModel for PostRow {
    type PrimaryKey = (Uuid,);
    type PartitionKey = (Uuid,);

    const TABLE_NAME: &'static str = "posts";
    const FIND_ALL_QUERY: &'static str = PostRow::FIND_ALL_QUERY;
    const FIND_BY_PRIMARY_KEY_QUERY: &'static str = PostRow::FIND_BY_PRIMARY_KEY_QUERY;
    const FIND_BY_PARTITION_KEY_QUERY: &'static str = PostRow::FIND_BY_PRIMARY_KEY_QUERY;

    fn primary_key_values(&self) -> (Uuid,) {
        (self.post_id,)
    }

    fn partition_key_values(&self) -> (Uuid,) {
        (self.post_id,)
    }
}

impl Model for PostRow {
    const INSERT_QUERY: &'static str = PostRow::INSERT_QUERY;
    const INSERT_IF_NOT_EXISTS_QUERY: &'static str = "INSERT INTO posts \
        (post_id, author_id, title, slug, content, status, tags, published_at, deleted_at, created_at, updated_at) \
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) IF NOT EXISTS";
    const UPDATE_QUERY: &'static str = PostRow::UPDATE_QUERY;
    const DELETE_QUERY: &'static str = PostRow::DELETE_QUERY;
    const DELETE_BY_PARTITION_KEY_QUERY: &'static str = PostRow::DELETE_QUERY;
}
