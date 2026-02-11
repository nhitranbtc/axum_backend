/// Integration tests for authentication endpoints
use crate::common::*;
use reqwest::StatusCode;
use serde_json::json;
use serial_test::serial;

// ============================================================================
// User Registration Tests
// ============================================================================

#[tokio::test]
#[serial]
async fn test_register_success() {
    let server = TestServer::new().await;
    let email = unique_email("reg_success");

    let res = server.register_user(&email, "Test User", TEST_PASSWORD).await;

    assert_success(&res);
    assert_eq!(res["data"]["user"]["email"], email);
    // Note: token might be empty in JSON if using cookies, check design
}

#[tokio::test]
#[serial]
async fn test_register_duplicate_email() {
    let server = TestServer::new().await;
    let email = unique_email("reg_dup");

    // First
    assert_success(&server.register_user(&email, "User 1", TEST_PASSWORD).await);

    // Second (Fail)
    let res = server
        .client
        .post(format!("{}/api/auth/register", server.base_url))
        .json(&json!({
            "email": email,
            "name": "User 2",
            "password": TEST_PASSWORD
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_error(&res);
}

#[tokio::test]
#[serial]
async fn test_register_invalid_email() {
    let server = TestServer::new().await;

    let res = server
        .client
        .post(format!("{}/api/auth/register", server.base_url))
        .json(&json!({
            "email": "not-an-email",
            "name": "Bad Email",
            "password": TEST_PASSWORD
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_error(&res);
}

#[tokio::test]
#[serial]
async fn test_set_password_weak_password() {
    let server = TestServer::new().await;
    let email = unique_email("weak_pass_flow");

    // 1. Register manually (helper panics on failure)
    let reg_res = server
        .client
        .post(format!("{}/api/auth/register", server.base_url))
        .json(&json!({
            "email": email,
            "name": "Weak Pass User"
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_success(&reg_res);

    // 2. Get Code
    let code = server.get_confirmation_code(&email).await;

    // 3. Set Weak Password
    let res = server
        .client
        .post(format!("{}/api/auth/password", server.base_url))
        .json(&json!({
            "email": email,
            "code": code,
            "password": "123" // Too short
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_error(&res);
}

// ============================================================================
// Login Tests
// ============================================================================

#[tokio::test]
#[serial]
async fn test_login_success() {
    let server = TestServer::new().await;
    let email = unique_email("login_ok");

    server.register_user(&email, "Login User", TEST_PASSWORD).await;

    let token = server.login_user(&email, TEST_PASSWORD).await;
    assert!(!token.is_empty());
}

#[tokio::test]
#[serial]
async fn test_login_wrong_credentials() {
    let server = TestServer::new().await;
    let email = unique_email("login_bad");

    server.register_user(&email, "Login User", TEST_PASSWORD).await;

    // Wrong Password
    let res = server
        .client
        .post(format!("{}/api/auth/login", server.base_url))
        .json(&json!({ "email": email, "password": "WrongPassword" }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_error(&res);

    // Wrong Email
    let res = server
        .client
        .post(format!("{}/api/auth/login", server.base_url))
        .json(&json!({ "email": "nothere@test.com", "password": TEST_PASSWORD }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_error(&res);
}

// ============================================================================
// Protected Resources
// ============================================================================

#[tokio::test]
#[serial]
async fn test_list_users() {
    let server = TestServer::new().await;
    let email = unique_email("list_users");

    server.register_user(&email, "List User", TEST_PASSWORD).await;
    let token = server.login_user(&email, TEST_PASSWORD).await;

    // 1. Authenticated (via Bearer Token)
    let res = server.list_users(&token, 1, 10).await;
    assert_success(&res);
    assert!(res["data"].is_array());

    // 2. Authenticated (via Cookie in server.client)
    let res_cookie = server
        .client
        .get(format!("{}/api/users", server.base_url))
        .query(&[("page", "1"), ("page_size", "10")])
        .send()
        .await
        .unwrap();
    assert_eq!(res_cookie.status(), StatusCode::OK);

    // 3. Unauthenticated (Fresh Client)
    let fresh_client = reqwest::Client::new();
    let res_unauth =
        fresh_client.get(format!("{}/api/users", server.base_url)).send().await.unwrap();

    assert_eq!(res_unauth.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[serial]
async fn test_concurrent_registrations() {
    let server = TestServer::new().await;
    let mut handles = vec![];

    for i in 0..5 {
        let server_url = server.base_url.clone();
        let client = server.client.clone();

        handles.push(tokio::spawn(async move {
            let email = unique_email(&format!("concurrent_{}", i));
            client
                .post(format!("{}/api/auth/register", server_url))
                .json(&json!({
                    "email": email,
                    "name": format!("User {}", i),
                    "password": TEST_PASSWORD
                }))
                .send()
                .await
                .unwrap()
                .json::<serde_json::Value>()
                .await
                .unwrap()
        }));
    }

    for h in handles {
        let res = h.await.unwrap();
        assert_success(&res);
    }
}

// ============================================================================
// Forgot Password Tests
// ============================================================================

#[tokio::test]
#[serial]
async fn test_forgot_password_flow() {
    let server = TestServer::new().await;
    let email = unique_email("forgot_pass");
    let initial_password = "InitialPass123!";
    let new_password = "NewPass123!";

    // Step 1: Call Register
    server.register_user(&email, "Forgot User", initial_password).await;

    // Login to verify initial password works (optional verification)
    let token = server.login_user(&email, initial_password).await;
    assert!(!token.is_empty());

    // Step 2: Call Forgot Password
    let forgot_res = server
        .client
        .post(format!("{}/api/auth/forgot-password", server.base_url))
        .json(&json!({ "email": email }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_success(&forgot_res);

    // Get the confirmation code (simulating email retrieval)
    let code = server.get_confirmation_code(&email).await;
    assert_eq!(code.len(), 6);

    // Step 3: Call Set Password
    let set_pass_res = server
        .client
        .post(format!("{}/api/auth/password", server.base_url))
        .json(&json!({
            "email": email,
            "code": code,
            "password": new_password
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_success(&set_pass_res);

    // 5. Login with NEW password
    let new_token = server.login_user(&email, new_password).await;
    assert!(!new_token.is_empty());

    // 6. Login with OLD password should fail
    let old_login_res = server
        .client
        .post(format!("{}/api/auth/login", server.base_url))
        .json(&json!({ "email": email, "password": initial_password }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_error(&old_login_res);
}

// ============================================================================
// Resend Confirm Code Flow Tests
// ============================================================================

#[tokio::test]
#[serial]
async fn test_resend_code_flow() {
    let server = TestServer::new().await;
    // Step 1: Call Register
    let email = unique_email("resend_code");
    let name = "Resend User";
    // Note: Use simple regiter first part, not the helper that completes flow.
    let register_res = server
        .client
        .post(format!("{}/api/auth/register", server.base_url))
        .json(&json!({
            "email": email,
            "name": name
        }))
        .send()
        .await
        .unwrap();

    assert!(register_res.status().is_success());

    // Step 2: Call Reset Confirm Code (Resend Code)
    let resend_res = server
        .client
        .post(format!("{}/api/auth/resend-code", server.base_url))
        .json(&json!({ "email": email }))
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();

    assert_success(&resend_res);

    // Get the NEW code
    let code = server.get_confirmation_code(&email).await;
    assert_eq!(code.len(), 6);

    // Step 3: Call Verify
    let verify_res = server
        .client
        .post(format!("{}/api/auth/verify", server.base_url))
        .json(&json!({
            "email": email,
            "code": code
        }))
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>() // Use generic JSON because response might be simpler string wrapper
        .await
        .unwrap();

    assert_success(&verify_res);
}

// ============================================================================
// Full Authentication Flow Tests
// ============================================================================

#[tokio::test]
#[serial]
async fn test_full_auth_flow() {
    let server = TestServer::new().await;
    let email = unique_email("flow_test");
    let name = "Flow User";
    let password = TEST_PASSWORD;

    // 1. Register
    let res = server
        .client
        .post(format!("{}/api/auth/register", server.base_url))
        .json(&json!({
            "email": email,
            "name": name
        }))
        .send()
        .await
        .expect("Failed to send register request");

    assert_success(&res.json().await.unwrap());

    // 2. Get Code
    let code = server.get_confirmation_code(&email).await;
    assert_eq!(code.len(), 6);

    // 3. Verify Code
    let res = server
        .client
        .post(format!("{}/api/auth/verify", server.base_url))
        .json(&json!({
            "email": email,
            "code": code
        }))
        .send()
        .await
        .expect("Failed to send verify request");

    // Note: Verify endpoint returns simple 200 OK without body wrapping in some versions,
    // but based on TestServer helpers it seems to return JSON.
    assert_eq!(res.status(), StatusCode::OK);

    // 4. Set Password
    let res = server
        .client
        .post(format!("{}/api/auth/password", server.base_url))
        .json(&json!({
            "email": email,
            "code": code,
            "password": password
        }))
        .send()
        .await
        .expect("Failed to send set password request");

    assert_success(&res.json().await.unwrap());

    // 5. Login with Password
    let res = server
        .client
        .post(format!("{}/api/auth/login", server.base_url))
        .json(&json!({
            "email": email,
            "password": password
        }))
        .send()
        .await
        .expect("Failed to send login request");

    assert_eq!(res.status(), StatusCode::OK);

    // Check for cookie
    let cookie = res.cookies().find(|c| c.name() == "access_token");
    assert!(cookie.is_some(), "Access token cookie should be present");

    // 6. Login with Code should FAIL (code reused/cleared)
    let res = server
        .client
        .post(format!("{}/api/auth/login", server.base_url))
        .json(&json!({
            "email": email,
            "code": code
        }))
        .send()
        .await
        .expect("Failed to execute request");

    // Expect 401 Unauthorized or 400 Bad Request depending on implementation of strict code usage
    assert!(res.status().is_client_error());
}

#[tokio::test]
#[serial]
async fn test_login_with_code_flow() {
    let server = TestServer::new().await;
    let email = unique_email("code_login");
    let name = "Code User";

    // 1. Register
    let res = server
        .client
        .post(format!("{}/api/auth/register", server.base_url))
        .json(&json!({
            "email": email,
            "name": name
        }))
        .send()
        .await
        .expect("Failed to send register request");

    assert_success(&res.json().await.unwrap());

    // 2. Get Code
    let code = server.get_confirmation_code(&email).await;

    // 3. Verify Code
    let res = server
        .client
        .post(format!("{}/api/auth/verify", server.base_url))
        .json(&json!({
            "email": email,
            "code": code
        }))
        .send()
        .await
        .expect("Failed to send verify request");

    assert_eq!(res.status(), StatusCode::OK);

    // 4. Login with Code (Skip Set Password)
    // Code should still be valid because `VerifyEmail` does NOT clear it in some flows,
    // OR we might need to regenerat it.
    // Based on previous analysis: `Login` clears the code.

    let res = server
        .client
        .post(format!("{}/api/auth/login", server.base_url))
        .json(&json!({
            "email": email,
            "code": code
        }))
        .send()
        .await
        .expect("Failed to send login request");

    assert_eq!(res.status(), StatusCode::OK);

    // Check for cookie
    let cookie = res.cookies().find(|c| c.name() == "access_token");
    assert!(cookie.is_some(), "Access token cookie should be present after code login");
}
