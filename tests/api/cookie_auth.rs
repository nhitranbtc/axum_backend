use crate::common::*;
use reqwest::StatusCode;
use serde_json::{json, Value};

#[tokio::test]
async fn test_cookie_auth_login_flow() {
    let server = TestServer::new().await;
    let email = unique_email("cookie_login");

    // 1. Register
    server.register_user(&email, "Login User", TEST_PASSWORD).await;

    // 2. Login (should set cookie)
    let login_res = server
        .client
        .post(format!("{}/api/auth/login", server.base_url))
        .json(&json!({
            "email": email,
            "password": TEST_PASSWORD
        }))
        .send()
        .await
        .expect("Failed to login");

    assert_eq!(login_res.status(), StatusCode::OK);

    let cookies: Vec<_> = login_res.cookies().collect();
    assert!(cookies.iter().any(|c| c.name() == "access_token"), "Login should set cookie");

    // 3. Access Protected Route
    let me_res = server
        .client
        .get(format!("{}/api/users", server.base_url)) // Assuming listing is protected
        .send()
        .await
        .unwrap();

    assert_eq!(me_res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_cookie_auth_logout() {
    let server = TestServer::new().await;
    let email = unique_email("cookie_logout");

    // Register & Login implicitly via register
    server.register_user(&email, "Logout User", TEST_PASSWORD).await;

    // Verify access
    let check_res = server.get_users_list_raw().await;
    assert_eq!(check_res.status(), StatusCode::OK);

    // Logout
    let logout_res = server
        .client
        .post(format!("{}/api/auth/logout", server.base_url))
        .json(&json!({ "logout_all": false }))
        .send()
        .await
        .expect("Failed to logout");

    assert_eq!(logout_res.status(), StatusCode::OK);

    // Verify Cookie Cleared (usually implies Set-Cookie with past expiry)
    let _cookies: Vec<_> = logout_res.cookies().collect();
    // In reqwest, cookies with past expiry might be removed from the jar automatically
    // or present with diff attributes. We check access.

    // Access should fail now
    let fail_res = server.get_users_list_raw().await;
    assert_eq!(fail_res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_cookie_auth_missing_cookie() {
    let server = TestServer::new().await;

    // Create a generic client WITHOUT cookie store for this test to simulate no cookies
    let raw_client = reqwest::Client::new();

    let res = raw_client.get(format!("{}/api/users", server.base_url)).send().await.unwrap();

    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

// Extension to TestServer for raw requests if needed
impl TestServer {
    pub async fn get_users_list_raw(&self) -> reqwest::Response {
        self.client
            .get(format!("{}/api/users", self.base_url))
            .query(&[("page", "1"), ("page_size", "10")])
            .send()
            .await
            .unwrap()
    }
}
