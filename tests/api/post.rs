use crate::common::*;
use reqwest::StatusCode;
use serde_json::{json, Value};
use serial_test::serial;

async fn create_post_with_token(server: &TestServer, token: &str, title: &str) -> Value {
    let response = server
        .client
        .post(format!("{}/api/posts", server.base_url))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({
            "title": title,
            "content": "Hello from integration test",
            "status": "draft",
            "tags": ["rust", "axum"]
        }))
        .send()
        .await
        .expect("Failed to send create post request");

    assert_eq!(response.status(), StatusCode::CREATED);
    let body: Value = response.json().await.expect("Failed to parse create post response");
    assert_success(&body);
    body
}

#[tokio::test]
#[serial]
async fn test_create_post_success_with_bearer_token() {
    let server = TestServer::new().await;
    let email = unique_email("post_create_ok");

    server.register_user(&email, "Post Author", TEST_PASSWORD).await;
    let token = server.login_user(&email, TEST_PASSWORD).await;

    let body = create_post_with_token(&server, &token, "First Post").await;
    assert_success(&body);
    assert_eq!(body["data"]["title"], "First Post");
    assert_eq!(body["data"]["content"], "Hello from integration test");
    assert_eq!(body["data"]["status"], "draft");
    assert!(body["data"]["slug"].as_str().is_some_and(|v| !v.is_empty()));
    assert!(body["data"]["id"].as_str().is_some_and(|v| !v.is_empty()));
    assert!(body["data"]["author_id"].as_str().is_some_and(|v| !v.is_empty()));
}

#[tokio::test]
#[serial]
async fn test_create_post_requires_authentication() {
    let server = TestServer::new().await;

    let response = server
        .client
        .post(format!("{}/api/posts", server.base_url))
        .json(&json!({
            "title": "Unauthorized Post",
            "content": "Should fail"
        }))
        .send()
        .await
        .expect("Failed to send create post request");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[serial]
async fn test_create_post_validation_error() {
    let server = TestServer::new().await;
    let email = unique_email("post_create_bad");

    server.register_user(&email, "Post Author", TEST_PASSWORD).await;
    let token = server.login_user(&email, TEST_PASSWORD).await;

    let response = server
        .client
        .post(format!("{}/api/posts", server.base_url))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({
            "title": "",
            "content": "Has content but empty title"
        }))
        .send()
        .await
        .expect("Failed to send create post request");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body: Value = response.json().await.expect("Failed to parse validation error response");
    assert!(body.get("error").is_some(), "Expected error field in response body: {body:?}");
}

#[tokio::test]
#[serial]
async fn test_get_post_success() {
    let server = TestServer::new().await;
    let email = unique_email("post_get_ok");
    server.register_user(&email, "Post Author", TEST_PASSWORD).await;
    let token = server.login_user(&email, TEST_PASSWORD).await;

    let created = create_post_with_token(&server, &token, "Get Post").await;
    let post_id = created["data"]["id"].as_str().unwrap();

    let response = server
        .client
        .get(format!("{}/api/posts/{}", server.base_url, post_id))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("Failed to send get post request");

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = response.json().await.expect("Failed to parse get post response");
    assert_success(&body);
    assert_eq!(body["data"]["id"], post_id);
}

#[tokio::test]
#[serial]
async fn test_get_post_not_found() {
    let server = TestServer::new().await;
    let email = unique_email("post_get_404");
    server.register_user(&email, "Post Author", TEST_PASSWORD).await;
    let token = server.login_user(&email, TEST_PASSWORD).await;

    let response = server
        .client
        .get(format!("{}/api/posts/{}", server.base_url, uuid::Uuid::new_v4()))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("Failed to send get post request");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
#[serial]
async fn test_list_posts_with_page_page_size_status() {
    let server = TestServer::new().await;
    let email = unique_email("post_list_ok");
    server.register_user(&email, "Post Author", TEST_PASSWORD).await;
    let token = server.login_user(&email, TEST_PASSWORD).await;

    create_post_with_token(&server, &token, "List Post 1").await;
    create_post_with_token(&server, &token, "List Post 2").await;

    let response = server
        .client
        .get(format!("{}/api/posts", server.base_url))
        .header("Authorization", format!("Bearer {token}"))
        .query(&[("page", "1"), ("page_size", "10"), ("status", "draft")])
        .send()
        .await
        .expect("Failed to send list posts request");

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = response.json().await.expect("Failed to parse list posts response");
    assert_success(&body);
    assert!(body["data"]["items"].is_array());
    assert!(body["data"]["total_items"].as_i64().unwrap_or(0) >= 2);
}

#[tokio::test]
#[serial]
async fn test_update_post_authorized() {
    let server = TestServer::new().await;
    let email = unique_email("post_update_ok");
    server.register_user(&email, "Post Owner", TEST_PASSWORD).await;
    let token = server.login_user(&email, TEST_PASSWORD).await;

    let created = create_post_with_token(&server, &token, "Old Title").await;
    let post_id = created["data"]["id"].as_str().unwrap();

    let response = server
        .client
        .put(format!("{}/api/posts/{}", server.base_url, post_id))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({
            "title": "New Title",
            "content": "Updated content",
            "status": "published",
            "tags": ["updated", "post"]
        }))
        .send()
        .await
        .expect("Failed to send update post request");

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = response.json().await.expect("Failed to parse update post response");
    assert_success(&body);
    assert_eq!(body["data"]["title"], "New Title");
    assert_eq!(body["data"]["status"], "published");

    // Regression check for cache invalidation/read freshness:
    // GET should return the updated values after PUT.
    let get_response = server
        .client
        .get(format!("{}/api/posts/{}", server.base_url, post_id))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("Failed to send get post request after update");

    assert_eq!(get_response.status(), StatusCode::OK);
    let get_body: Value = get_response
        .json()
        .await
        .expect("Failed to parse get post response after update");
    assert_success(&get_body);
    assert_eq!(get_body["data"]["title"], "New Title");
    assert_eq!(get_body["data"]["content"], "Updated content");
}

#[tokio::test]
#[serial]
async fn test_update_post_forbidden_for_non_owner() {
    let server = TestServer::new().await;

    let owner_email = unique_email("post_owner");
    server.register_user(&owner_email, "Post Owner", TEST_PASSWORD).await;
    let owner_token = server.login_user(&owner_email, TEST_PASSWORD).await;
    let created = create_post_with_token(&server, &owner_token, "Owner Post").await;
    let post_id = created["data"]["id"].as_str().unwrap();

    let other_email = unique_email("post_other");
    server.register_user(&other_email, "Other User", TEST_PASSWORD).await;
    let other_token = server.login_user(&other_email, TEST_PASSWORD).await;

    let response = server
        .client
        .put(format!("{}/api/posts/{}", server.base_url, post_id))
        .header("Authorization", format!("Bearer {other_token}"))
        .json(&json!({ "title": "Hacked Title" }))
        .send()
        .await
        .expect("Failed to send update post request");

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
#[serial]
async fn test_delete_post_soft_delete() {
    let server = TestServer::new().await;
    let email = unique_email("post_delete_ok");
    server.register_user(&email, "Post Author", TEST_PASSWORD).await;
    let token = server.login_user(&email, TEST_PASSWORD).await;

    let created = create_post_with_token(&server, &token, "Delete Me").await;
    let post_id = created["data"]["id"].as_str().unwrap();

    let delete_response = server
        .client
        .delete(format!("{}/api/posts/{}", server.base_url, post_id))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("Failed to send delete post request");

    assert_eq!(delete_response.status(), StatusCode::OK);

    let get_response = server
        .client
        .get(format!("{}/api/posts/{}", server.base_url, post_id))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("Failed to send get post request after delete");

    assert_eq!(get_response.status(), StatusCode::NOT_FOUND);
}
