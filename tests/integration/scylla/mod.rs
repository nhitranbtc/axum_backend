//! ScyllaDB integration tests using testcontainers
//!
//! These tests spin up a real ScyllaDB container automatically via Docker.
//! No external ScyllaDB instance is required.
//!
//! Run with:
//!   cargo test --test scylla_integration_test
//!
//! Prerequisites: Docker must be running on the host.

pub mod post;

use crate::common::mock::MockScylla;
use anyhow::Result;
use axum_backend::{
    config::scylla::ScyllaConfig,
    domain::{
        entities::refresh_token::RefreshToken,
        repositories::{AuthRepository, AuthRepositoryError},
    },
    infrastructure::database::scylla::{
        create_scylla_session, AuthRepositoryImpl, EventRepository, ScyllaSession,
        SessionRepository, UserEventRow, UserRepositoryImpl, UserSessionRow,
    },
};
use serial_test::serial;
use std::sync::Arc;
use tokio::sync::OnceCell;
use uuid::Uuid;

/// Start a ScyllaDB container and return an Arc<ScyllaSession> connected to it.
/// The container is kept alive globally using a OnceCell.
static ONCE_MOCK_SCYLLA: OnceCell<MockScylla> = OnceCell::const_new();

async fn start_scylla() -> Result<Arc<ScyllaSession>> {
    let mock_db = ONCE_MOCK_SCYLLA.get_or_init(|| async { MockScylla::new().await }).await;

    let config = ScyllaConfig {
        nodes: vec![mock_db.contact_node.clone()],
        keyspace: format!("test_keyspace_{}", uuid::Uuid::new_v4().simple()),
        username: Some(
            std::env::var("SCYLLA_USERNAME").unwrap_or_else(|_| "cassandra".to_string()),
        ),
        password: Some(
            std::env::var("SCYLLA_PASSWORD").unwrap_or_else(|_| "cassandra".to_string()),
        ),
        replication_factor: 1,
    };

    let session = Arc::new(create_scylla_session(&config).await?);
    Ok(session)
}

// ── Connection tests ──────────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn test_scylla_connection() {
    let session = start_scylla().await.expect("Failed to start ScyllaDB container");
    let health = session.health_check().await;
    assert!(health.is_ok(), "Health check failed: {:?}", health.err());
}

// ── UserRepository tests ──────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn test_user_repository_operations() {
    use axum_backend::domain::{
        entities::User, repositories::user_repository::UserRepository, value_objects::Email,
    };

    let session = start_scylla().await.expect("Failed to start container");
    let repo = UserRepositoryImpl::new(Arc::clone(&session));

    let email = Email::parse("test-save@example.com").unwrap();
    let user = User::new(email.clone(), "Test User".to_string()).unwrap();
    let user_id = user.id.clone();

    let saved = repo.save(&user).await.expect("save failed");
    assert_eq!(saved.id.as_uuid(), user_id.as_uuid());

    let found = repo.find_by_id(user_id.clone()).await.expect("find_by_id failed");
    assert!(found.is_some(), "User should be found by id");
    let found = found.unwrap();
    assert_eq!(found.email.as_str(), "test-save@example.com");

    // --- merged user repo test ---

    let email = Email::parse("find-by-email@example.com").unwrap();
    let user = User::new(email.clone(), "Find Email User".to_string()).unwrap();

    repo.save(&user).await.expect("save failed");

    let found = repo.find_by_email(&email).await.expect("find_by_email failed");
    assert!(found.is_some(), "User should be found by email");

    // --- merged user repo test ---

    let email = Email::parse("exists@example.com").unwrap();
    let user = User::new(email.clone(), "Exists User".to_string()).unwrap();

    assert!(!repo.exists_by_email(&email).await.unwrap(), "should not exist yet");
    repo.save(&user).await.expect("save failed");
    assert!(repo.exists_by_email(&email).await.unwrap(), "should exist after save");

    // --- merged user repo test ---

    let email = Email::parse("update@example.com").unwrap();
    let mut user = User::new(email.clone(), "Original Name".to_string()).unwrap();
    repo.save(&user).await.expect("save failed");

    user.name = "Updated Name".to_string();
    let updated = repo.update(&user).await.expect("update failed");
    assert_eq!(updated.name, "Updated Name");

    // --- merged user repo test ---

    let email = Email::parse("delete@example.com").unwrap();
    let user = User::new(email, "Delete User".to_string()).unwrap();
    let user_id = user.id.clone();

    repo.save(&user).await.expect("save failed");
    let deleted = repo.delete(user_id.clone()).await.expect("delete failed");
    assert!(deleted, "delete should return true");

    let found = repo.find_by_id(user_id).await.expect("find after delete failed");
    assert!(found.is_none(), "User should not be found after deletion");

    // --- merged user repo test ---

    // Ensure fresh keyspace (count might not be 0 if other tests ran in same container)
    for i in 0..3 {
        let email = Email::parse(&format!("paginate-{}@example.com", i)).unwrap();
        let user = User::new(email, format!("User {}", i)).unwrap();
        repo.save(&user).await.expect("save failed");
    }

    let count = repo.count().await.expect("count failed");
    assert!(count >= 3, "expected at least 3 users, got {}", count);

    let page = repo.list_paginated(2, 0).await.expect("list_paginated failed");
    assert_eq!(page.len(), 2, "expected 2 users in page");
}

// ── AuthRepository tests ──────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn test_auth_repository_operations() {
    let session = start_scylla().await.expect("Failed to start container");
    let repo = AuthRepositoryImpl::new(Arc::clone(&session));

    let email = "auth-create@example.com";
    let user = repo
        .create_user(email, "Auth User", Some("hashed_password".to_string()), None, None)
        .await
        .expect("create_user failed");

    assert_eq!(user.email.as_str(), email);

    let found = repo.find_by_email(email).await.expect("find_by_email failed");
    assert!(found.is_some(), "User should be found after creation");
    assert_eq!(found.unwrap().email.as_str(), email);

    // --- merged auth repo test ---

    let email = "duplicate@example.com";
    repo.create_user(email, "User One", None, None, None)
        .await
        .expect("first create should succeed");

    let result = repo.create_user(email, "User Two", None, None, None).await;
    assert!(
        matches!(result, Err(AuthRepositoryError::EmailAlreadyExists)),
        "second create should fail with EmailAlreadyExists, got {:?}",
        result
    );

    // --- merged auth repo test ---

    let user = repo
        .create_user("last-login@example.com", "Login User", None, None, None)
        .await
        .expect("create_user failed");

    let result = repo.update_last_login(*user.id.as_uuid()).await;
    assert!(result.is_ok(), "update_last_login failed: {:?}", result.err());

    // --- merged auth repo test ---

    let mut user = repo
        .create_user("update-user@example.com", "Old Name", None, None, None)
        .await
        .expect("create_user failed");

    user.name = "New Name".to_string();
    let updated = repo.update_user(&user).await.expect("update_user failed");
    assert_eq!(updated.name, "New Name");

    // ── Refresh token tests ───────────────────────────────────────────────────────
    // --- merged auth repo test ---

    let user = repo
        .create_user("token-lifecycle@example.com", "Token User", None, None, None)
        .await
        .expect("create_user failed");

    let expires = chrono::Utc::now() + chrono::Duration::days(7);
    let token = RefreshToken::new(*user.id.as_uuid(), "test_token_hash_abc".to_string(), expires);

    // Save
    repo.save_refresh_token(&token).await.expect("save_refresh_token failed");

    // Find
    let found = repo
        .find_refresh_token(&token.token_hash)
        .await
        .expect("find_refresh_token failed");
    assert!(found.is_some(), "Token should exist after save");
    assert!(found.unwrap().is_valid(), "Token should be valid");

    // Revoke
    repo.revoke_refresh_token(&token.token_hash)
        .await
        .expect("revoke_refresh_token failed");

    // Verify revoked
    let after_revoke = repo
        .find_refresh_token(&token.token_hash)
        .await
        .expect("find after revoke failed");
    assert!(after_revoke.is_some());
    assert!(!after_revoke.unwrap().is_valid(), "Token should be invalid after revoke");

    // --- merged auth repo test ---

    let user = repo
        .create_user("revoke-all@example.com", "Revoke All User", None, None, None)
        .await
        .expect("create_user failed");

    let user_id = *user.id.as_uuid();
    let expires = chrono::Utc::now() + chrono::Duration::days(7);

    for i in 0..3 {
        let token = RefreshToken::new(user_id, format!("token_hash_{}", i), expires);
        repo.save_refresh_token(&token).await.expect("save token failed");
    }

    repo.revoke_all_user_tokens(user_id).await.expect("revoke_all failed");

    // Verify all 3 tokens are revoked
    for i in 0..3 {
        let hash = format!("token_hash_{}", i);
        let token = repo.find_refresh_token(&hash).await.expect("find failed");
        if let Some(t) = token {
            assert!(!t.is_valid(), "token {} should be revoked", i);
        }
    }
}

// ── EventRepository tests ─────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn test_event_repository_operations() {
    let session = start_scylla().await.expect("Failed to start container");
    let repo = Arc::new(EventRepository::new(Arc::clone(&session)));

    let user_id = Uuid::new_v4();
    let event =
        UserEventRow::new(user_id, "test.event".to_string(), r#"{"action": "test"}"#.to_string());

    let result = repo.save_event(&event).await;
    assert!(result.is_ok(), "save_event failed: {:?}", result.err());

    // --- merged event repo test ---
    let repo = Arc::new(EventRepository::new(Arc::clone(&session)));

    let user_id = Uuid::new_v4();
    for event_type in ["user.login", "user.profile_update", "user.logout"] {
        let event = UserEventRow::new(
            user_id,
            event_type.to_string(),
            format!(r#"{{"action": "{}"}}"#, event_type),
        );
        repo.save_event(&event).await.expect("save_event failed");
    }

    // --- merged event repo test ---
    let repo = Arc::new(EventRepository::new(Arc::clone(&session)));

    let user_id = Uuid::new_v4();
    let mut handles = vec![];

    for i in 0..10 {
        let repo_clone: Arc<EventRepository> = Arc::clone(&repo);
        let handle = tokio::spawn(async move {
            let event = UserEventRow::new(
                user_id,
                format!("concurrent.{}", i),
                format!(r#"{{"index": {}}}"#, i),
            );
            repo_clone.save_event(&event).await
        });
        handles.push(handle);
    }

    for handle in handles {
        let result = handle.await.expect("Task panicked");
        assert!(result.is_ok(), "concurrent write failed: {:?}", result.err());
    }
}

// ── SessionRepository tests ───────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn test_session_repository_save_and_delete() {
    let session = start_scylla().await.expect("Failed to start container");
    let repo = SessionRepository::new(Arc::clone(&session));

    let user_id = Uuid::new_v4();
    let session_data = UserSessionRow::new(
        user_id,
        r#"{"user_agent": "test", "ip": "127.0.0.1"}"#.to_string(),
        3600,
    );

    repo.save_session(&session_data).await.expect("save_session failed");

    let result = repo.delete_session(session_data.session_id).await;
    assert!(result.is_ok(), "delete_session failed: {:?}", result.err());
}

// ── Pure unit tests (no container, no serial needed) ─────────────────────────

#[tokio::test]
async fn test_user_event_row_creation() {
    let user_id = Uuid::new_v4();
    let event = UserEventRow::new(user_id, "test.event".to_string(), "{}".to_string());

    assert_eq!(event.user_id, user_id);
    assert_eq!(event.event_type, "test.event");
    assert_ne!(event.event_id, uuid::Uuid::nil());
}

#[tokio::test]
async fn test_user_session_row_creation() {
    let user_id = Uuid::new_v4();
    let session = UserSessionRow::new(user_id, "{}".to_string(), 3600);

    assert_eq!(session.user_id, user_id);
    let expected = chrono::Utc::now() + chrono::Duration::seconds(3600);
    let expected_ms = expected.timestamp_millis();
    let diff = (session.expires_at.0 - expected_ms).abs();
    assert!(diff < 2000, "expiration should be within 2s of expected");
}

#[tokio::test]
async fn test_session_not_expired() {
    let user_id = Uuid::new_v4();
    let session = UserSessionRow::new(user_id, "{}".to_string(), 3600);
    assert!(!session.is_expired(), "fresh session should not be expired");
}

#[tokio::test]
async fn test_session_is_expired() {
    let user_id = Uuid::new_v4();
    // TTL of 1s then sleep
    let session_data = UserSessionRow::new(user_id, "{}".to_string(), 1);
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    assert!(session_data.is_expired(), "session should be expired after TTL");
}
