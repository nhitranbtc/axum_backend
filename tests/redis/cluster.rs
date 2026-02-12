use axum_backend::infrastructure::cache::{CacheRepository, RedisCacheRepository};
use std::time::Duration;
use testcontainers::{runners::AsyncRunner, GenericImage, ImageExt};

#[tokio::test]
#[ignore] // Ignored by default due to potential network complexity
async fn operations_cluster() {
    // Keep reference to container to ensuring it stays up
    let _container = GenericImage::new("grokzen/redis-cluster", "latest")
        .with_network("host") // Requires host networking for cluster to announce correct ports
        .with_env_var("IP", "0.0.0.0")
        .start()
        .await
        .expect("Failed to start Redis Cluster");

    // grokzen/redis-cluster exposes 7000-7005 by default
    let redis_url = "redis-cluster://127.0.0.1:7000";

    // Wait a bit for cluster to stabilize
    tokio::time::sleep(Duration::from_secs(5)).await;

    let repository = RedisCacheRepository::new(redis_url)
        .await
        .expect("Failed to create repository with cluster URL");

    // Test Set
    repository
        .set("cluster_key", "cluster_value", Duration::from_secs(60))
        .await
        .expect("Failed to set key in cluster");

    // Test Get
    let value = repository.get("cluster_key").await.expect("Failed to get key from cluster");
    assert_eq!(value, Some("cluster_value".to_string()));

    // Test Delete
    repository.delete("cluster_key").await.expect("Failed to delete key from cluster");
    let value = repository.get("cluster_key").await.expect("Failed to get key after delete");
    assert_eq!(value, None);
}
