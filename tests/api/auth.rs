//! Integration tests for authentication endpoints.
//!
//! Every test uses an ephemeral in-process server backed by a fresh
//! ScyllaDB keyspace, so tests are fully isolated and order-independent.
use crate::common::*;
use reqwest::StatusCode;
use serde_json::json;
use serial_test::serial;

// ============================================================================
// Registration
// ============================================================================

/// Happy-path registration returns 201 and echoes the email.
#[tokio::test]
#[serial]
async fn test_register_success() {
    let server = TestServer::new().await;
    let email = unique_email("reg_success");

    let res = server.register_user(&email, "Test User", TEST_PASSWORD).await;

    assert_success(&res);
    assert_eq!(res["data"]["user"]["email"], email);
}

/// A second registration with the same email must fail.
#[tokio::test]
#[serial]
async fn test_register_duplicate_email() {
    let server = TestServer::new().await;
    let email = unique_email("reg_dup");

    server.register_user(&email, "User 1", TEST_PASSWORD).await;

    let res = server
        .post_json(
            "/api/auth/register",
            json!({ "email": email, "name": "User 2", "password": TEST_PASSWORD }),
        )
        .await;

    assert_error(&res);
}

/// A malformed email address must be rejected at validation.
#[tokio::test]
#[serial]
async fn test_register_invalid_email() {
    let server = TestServer::new().await;

    let res = server
        .post_json(
            "/api/auth/register",
            json!({ "email": "not-an-email", "name": "Bad Email", "password": TEST_PASSWORD }),
        )
        .await;

    assert_error(&res);
}

/// A password shorter than the minimum length must be rejected.
#[tokio::test]
#[serial]
async fn test_set_password_too_short() {
    let server = TestServer::new().await;
    let email = unique_email("weak_pass");

    // Register without a password so we can call set-password with a bad one
    let reg = server
        .post_json("/api/auth/register", json!({ "email": email, "name": "Weak Pass User" }))
        .await;
    assert_success(&reg);

    let code = server.get_confirmation_code(&email).await;

    let res = server
        .post_json(
            "/api/auth/password",
            json!({ "email": email, "code": code, "password": "123" }),
        )
        .await;

    assert_error(&res);
}

// ============================================================================
// Login
// ============================================================================

/// Registered and verified users can log in and receive an access token.
#[tokio::test]
#[serial]
async fn test_login_success() {
    let server = TestServer::new().await;
    let email = unique_email("login_ok");

    server.register_user(&email, "Login User", TEST_PASSWORD).await;

    let token = server.login_user(&email, TEST_PASSWORD).await;
    assert!(!token.is_empty(), "Expected a non-empty access token");
}

/// Wrong password and non-existent email must both be rejected.
#[tokio::test]
#[serial]
async fn test_login_wrong_credentials() {
    let server = TestServer::new().await;
    let email = unique_email("login_bad");

    server.register_user(&email, "Login User", TEST_PASSWORD).await;

    let wrong_pass = server
        .post_json("/api/auth/login", json!({ "email": email, "password": "WrongPassword" }))
        .await;
    assert_error(&wrong_pass);

    let wrong_email = server
        .post_json(
            "/api/auth/login",
            json!({ "email": "nobody@test.com", "password": TEST_PASSWORD }),
        )
        .await;
    assert_error(&wrong_email);
}

// ============================================================================
// Protected resources
// ============================================================================

/// Verify three access scenarios for a protected endpoint:
/// authenticated with Bearer, authenticated via cookie session, unauthenticated.
#[tokio::test]
#[serial]
async fn test_list_users_access_control() {
    let server = TestServer::new().await;
    let email = unique_email("list_users");

    server.register_user(&email, "List User", TEST_PASSWORD).await;
    let token = server.login_user(&email, TEST_PASSWORD).await;

    // Bearer token — should succeed
    let res = server.list_users(&token, 1, 10).await;
    assert_success(&res);
    assert!(res["data"].is_array(), "Expected data to be an array");

    // Cookie session carried by server.client — should succeed
    let cookie_status = server
        .client
        .get(format!("{}/api/users", server.base_url))
        .query(&[("page", "1"), ("page_size", "10")])
        .send()
        .await
        .unwrap()
        .status();
    assert_eq!(cookie_status, StatusCode::OK);

    // Fresh client with no credentials — should be rejected
    let unauth_status = reqwest::Client::new()
        .get(format!("{}/api/users", server.base_url))
        .send()
        .await
        .unwrap()
        .status();
    assert_eq!(unauth_status, StatusCode::UNAUTHORIZED);
}

// ============================================================================
// Concurrent registrations
// ============================================================================

/// Five concurrent registrations with different emails must all succeed.
#[tokio::test]
#[serial]
async fn test_concurrent_registrations() {
    let server = TestServer::new().await;

    let handles: Vec<_> = (0..5)
        .map(|i| {
            let url = server.base_url.clone();
            let client = server.client.clone();
            tokio::spawn(async move {
                let email = unique_email(&format!("concurrent_{i}"));
                client
                    .post(format!("{url}/api/auth/register"))
                    .json(&json!({
                        "email": email,
                        "name": format!("User {i}"),
                        "password": TEST_PASSWORD,
                    }))
                    .send()
                    .await
                    .expect("concurrent register send failed")
                    .json::<serde_json::Value>()
                    .await
                    .expect("concurrent register parse failed")
            })
        })
        .collect();

    for handle in handles {
        let res = handle.await.expect("task panicked");
        assert_success(&res);
    }
}

// ============================================================================
// Forgot-password flow
// ============================================================================

/// Full forgot-password cycle:
/// register → login with initial password → forgot-password → set new password
/// → login succeeds with new password → login fails with old password.
#[tokio::test]
#[serial]
async fn test_forgot_password_flow() {
    let server = TestServer::new().await;
    let email = unique_email("forgot_pass");
    let initial = "InitialPass123!";
    let updated = "NewPass123!";

    server.register_user(&email, "Forgot User", initial).await;

    // Baseline: initial password works
    let token = server.login_user(&email, initial).await;
    assert!(!token.is_empty());

    // Request a reset code and set the new password
    server.forgot_password(&email).await;
    let code = server.get_confirmation_code(&email).await;
    assert_eq!(code.len(), 6);
    server.set_password(&email, &code, updated).await;

    // New password must work
    let new_token = server.login_user(&email, updated).await;
    assert!(!new_token.is_empty());

    // Old password must no longer work
    let old_attempt = server
        .post_json("/api/auth/login", json!({ "email": email, "password": initial }))
        .await;
    assert_error(&old_attempt);
}

// ============================================================================
// Resend-code flow
// ============================================================================

/// After registration, resending the confirmation code and verifying with the
/// new code must succeed.
#[tokio::test]
#[serial]
async fn test_resend_code_flow() {
    let server = TestServer::new().await;
    let email = unique_email("resend_code");

    // Register without a password to exercise the code-based path
    let reg = server
        .post_json("/api/auth/register", json!({ "email": email, "name": "Resend User" }))
        .await;
    assert_success(&reg);

    // Resend a fresh code
    let resend = server
        .post_json("/api/auth/resend-code", json!({ "email": email }))
        .await;
    assert_success(&resend);

    // Verify with the (stub) code
    let code = server.get_confirmation_code(&email).await;
    assert_eq!(code.len(), 6);
    server.verify_email(&email, &code).await;
}

// ============================================================================
// Full auth flow
// ============================================================================

/// Complete auth lifecycle using the password-at-registration flow:
/// register (with password) → verify → login with password → code login.
#[tokio::test]
#[serial]
async fn test_full_auth_flow() {
    let server = TestServer::new().await;
    let email = unique_email("full_flow");

    // 1. Register with password
    let reg = server
        .post_json(
            "/api/auth/register",
            json!({ "email": email, "name": "Flow User", "password": TEST_PASSWORD }),
        )
        .await;
    assert_success(&reg);

    // 2. Verify email
    let code = server.get_confirmation_code(&email).await;
    assert_eq!(code.len(), 6);
    server.verify_email(&email, &code).await;

    // 3. Login with password — access_token cookie must be set
    let login_resp = server
        .client
        .post(format!("{}/api/auth/login", server.base_url))
        .json(&json!({ "email": email, "password": TEST_PASSWORD }))
        .send()
        .await
        .expect("login request failed");

    assert_eq!(login_resp.status(), StatusCode::OK);
    assert!(
        login_resp.cookies().any(|c| c.name() == "access_token"),
        "Expected access_token cookie after password login"
    );

    // 4. Code was consumed by verify — reusing it must fail
    let stale_code_resp = server
        .client
        .post(format!("{}/api/auth/login", server.base_url))
        .json(&json!({ "email": email, "code": code }))
        .send()
        .await
        .expect("stale code login request failed");

    assert!(
        stale_code_resp.status().is_client_error(),
        "Expected 4xx for stale confirmation code, got {}",
        stale_code_resp.status()
    );
}

// ============================================================================
// Code-based login
// ============================================================================

/// Users can log in with a fresh confirmation code instead of a password.
/// The code must be consumed (invalidated) after a successful login.
#[tokio::test]
#[serial]
async fn test_login_with_code_flow() {
    let server = TestServer::new().await;
    let email = unique_email("code_login");

    // Register without password to keep the code active after verify
    let reg = server
        .post_json("/api/auth/register", json!({ "email": email, "name": "Code User" }))
        .await;
    assert_success(&reg);

    let code = server.get_confirmation_code(&email).await;
    server.verify_email(&email, &code).await;

    // Login with the code (before it is cleared by a password-login)
    let login_resp = server
        .client
        .post(format!("{}/api/auth/login", server.base_url))
        .json(&json!({ "email": email, "code": code }))
        .send()
        .await
        .expect("code login request failed");

    assert_eq!(login_resp.status(), StatusCode::OK);
    assert!(
        login_resp.cookies().any(|c| c.name() == "access_token"),
        "Expected access_token cookie after code login"
    );
}
