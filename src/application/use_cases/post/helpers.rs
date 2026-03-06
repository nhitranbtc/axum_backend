use crate::{
    domain::{
        entities::PostStatus,
        repositories::{post::PostRepository, user_repository::UserRepository},
        value_objects::UserId,
    },
    shared::AppError,
};
use uuid::Uuid;

pub async fn ensure_actor_can_manage_post<UR: UserRepository>(
    user_repository: &UR,
    actor_id: Uuid,
    author_id: Uuid,
) -> Result<(), AppError> {
    let actor = user_repository
        .find_by_id(UserId::from_uuid(actor_id))
        .await?
        .ok_or_else(|| AppError::Unauthorized("User not found".to_string()))?;

    let is_author = author_id == actor_id;
    let is_admin = actor.role.can_delete();
    if !is_author && !is_admin {
        return Err(AppError::Forbidden);
    }

    Ok(())
}

pub async fn generate_unique_slug<PR: PostRepository>(
    post_repository: &PR,
    title: &str,
    current_post_id: Option<Uuid>,
) -> Result<String, AppError> {
    let base = slugify(title);
    if base.is_empty() {
        return Err(AppError::Validation("Title cannot produce a valid slug".to_string()));
    }

    if let Some(existing) = post_repository.find_by_slug(&base).await? {
        if Some(existing.id) != current_post_id {
            for i in 2..=10_000 {
                let candidate = format!("{}-{}", base, i);
                if let Some(found) = post_repository.find_by_slug(&candidate).await? {
                    if Some(found.id) == current_post_id {
                        return Ok(candidate);
                    }
                } else {
                    return Ok(candidate);
                }
            }
            return Err(AppError::Internal(anyhow::anyhow!("Failed to generate unique slug")));
        }
    } else {
        return Ok(base);
    }

    Ok(base)
}

pub fn parse_status(value: Option<&str>) -> Result<PostStatus, AppError> {
    if let Some(raw) = value {
        PostStatus::parse(raw)
            .ok_or_else(|| AppError::Validation("Invalid status value".to_string()))
    } else {
        Ok(PostStatus::Draft)
    }
}

fn slugify(input: &str) -> String {
    let mut slug = String::with_capacity(input.len());
    let mut prev_dash = false;

    for ch in input.chars().flat_map(|c| c.to_lowercase()) {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch);
            prev_dash = false;
        } else if !prev_dash {
            slug.push('-');
            prev_dash = true;
        }
    }

    slug.trim_matches('-').to_string()
}
