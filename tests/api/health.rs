use crate::common::*;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_health_check() {
    let server = TestServer::new().await;
    let response = server.health_check().await;

    assert_eq!(response.status(), 200);
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["status"], "healthy");
}
