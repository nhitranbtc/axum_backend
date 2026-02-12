use axum_backend::infrastructure::cache::{CacheRepository, RedisCacheRepository};
use std::time::Duration;
use testcontainers::{core::ContainerPort, runners::AsyncRunner, GenericImage};

#[tokio::test]
async fn operations() {
    let container = GenericImage::new("redis", "7.2-alpine")
        .with_exposed_port(ContainerPort::Tcp(6379))
        .start()
        .await
        .expect("Failed to start Redis");

    let host_port = container.get_host_port_ipv4(6379).await.expect("Failed to get port");
    let redis_url = format!("redis://127.0.0.1:{}", host_port);

    let repository = RedisCacheRepository::new(&redis_url)
        .await
        .expect("Failed to create repository");

    // Test Set
    repository
        .set("test_key", "test_value", Duration::from_secs(60))
        .await
        .expect("Failed to set key");

    // Test Get
    let value = repository.get("test_key").await.expect("Failed to get key");
    assert_eq!(value, Some("test_value".to_string()));

    // Test Delete
    repository.delete("test_key").await.expect("Failed to delete key");
    let value = repository.get("test_key").await.expect("Failed to get key");
    assert_eq!(value, None);
}
