use crate::domain::entities::Post;
use std::time::Duration;
use uuid::Uuid;

const POST_CACHE_TTL_SECONDS: u64 = 3600;

pub fn post_key(id: Uuid) -> String {
    format!("post:{id}")
}

pub fn post_slug_key(slug: &str) -> String {
    format!("post:slug:{slug}")
}

pub fn cache_ttl() -> Duration {
    Duration::from_secs(POST_CACHE_TTL_SECONDS)
}

pub fn serialize_post(post: &Post) -> Result<String, serde_json::Error> {
    serde_json::to_string(post)
}

pub fn deserialize_post(value: &str) -> Result<Post, serde_json::Error> {
    serde_json::from_str(value)
}
