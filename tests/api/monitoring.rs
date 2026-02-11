use crate::common::*;
use reqwest::StatusCode;

#[tokio::test]
async fn test_metrics_endpoint() {
    let server = TestServer::new().await;

    // Make a request to generate metrics
    let _ = server.client.get(format!("{}/health", server.base_url)).send().await;

    let response = server
        .client
        .get(format!("{}/metrics", server.base_url))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), StatusCode::OK);

    let text = response.text().await.expect("Failed to get response text");
    assert!(text.contains("# TYPE"), "Metrics output should contain Prometheus type");
    assert!(
        text.contains("axum_http_requests_total"),
        "Metrics output should contain request total"
    );
}

#[tokio::test]
async fn test_system_health_endpoint() {
    let server = TestServer::new().await;

    let response = server
        .client
        .get(format!("{}/api/admin/system", server.base_url))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), StatusCode::OK);

    let json: serde_json::Value = response.json().await.expect("Failed to parse JSON");

    // Check fields
    assert!(json.get("cpu_usage").is_some());
    assert!(json.get("total_memory").is_some());
    assert!(json.get("used_memory").is_some());
    assert!(json.get("uptime").is_some());

    // Basic sanity checks
    let total_mem = json["total_memory"].as_u64().expect("total_memory is not u64");
    assert!(total_mem > 0, "Total memory should be positive");
}
