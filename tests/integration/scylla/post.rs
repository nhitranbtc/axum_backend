use super::start_scylla;
use axum_backend::{
    domain::{
        entities::{Post, PostStatus},
        repositories::post::PostRepository,
    },
    infrastructure::database::scylla::PostRepositoryImpl,
};
use serial_test::serial;
use std::sync::Arc;
use uuid::Uuid;

#[tokio::test]
#[serial]
async fn test_post_repository_save_find_and_slug_lookup() {
    let session = start_scylla().await.expect("Failed to start container");
    let repo = PostRepositoryImpl::new(Arc::clone(&session));

    let author_id = Uuid::new_v4();
    let post = Post::new(
        author_id,
        "Scylla Post".to_string(),
        "scylla-post".to_string(),
        "content".to_string(),
        PostStatus::Draft,
        vec!["rust".to_string()],
    )
    .expect("Failed to create post entity");

    let saved = repo.save(&post).await.expect("save failed");
    let by_id = repo.find_by_id(saved.id).await.expect("find_by_id failed");
    assert!(by_id.is_some(), "expected post by id");

    let by_slug = repo.find_by_slug("scylla-post").await.expect("find_by_slug failed");
    assert!(by_slug.is_some(), "expected post by slug");
}

#[tokio::test]
#[serial]
async fn test_post_repository_update_list_and_soft_delete() {
    let session = start_scylla().await.expect("Failed to start container");
    let repo = PostRepositoryImpl::new(Arc::clone(&session));

    let author_id = Uuid::new_v4();
    let mut post = Post::new(
        author_id,
        "Original".to_string(),
        "original".to_string(),
        "content".to_string(),
        PostStatus::Draft,
        vec!["one".to_string()],
    )
    .expect("Failed to create post entity");

    let saved = repo.save(&post).await.expect("save failed");
    post = saved;
    post.update_title_and_slug("Updated".to_string(), "updated".to_string())
        .expect("update title failed");
    post.update_status(PostStatus::Published).expect("update status failed");

    let updated = repo.update(&post).await.expect("update failed");
    assert_eq!(updated.title, "Updated");
    assert_eq!(updated.status.as_str(), "published");

    let listed_by_author = repo
        .list_by_author(author_id, None, 10, 0)
        .await
        .expect("list_by_author failed");
    assert!(
        listed_by_author.iter().any(|p| p.id == post.id),
        "expected updated post in list_by_author"
    );

    let listed_recent =
        repo.list_recent(Some("published"), 10, 0).await.expect("list_recent failed");
    assert!(listed_recent.iter().any(|p| p.id == post.id), "expected post in list_recent");

    let deleted = repo.soft_delete(post.id).await.expect("soft_delete failed");
    assert!(deleted, "soft_delete should return true");

    let after_delete = repo.find_by_id(post.id).await.expect("find_by_id failed");
    assert!(after_delete.is_some(), "row still exists after soft delete");
    assert!(after_delete.unwrap().is_deleted(), "post should be marked as soft-deleted");
}
