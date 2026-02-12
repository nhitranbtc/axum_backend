use axum_backend::infrastructure::cache::{DistributedLock, RedisCacheRepository};
use std::sync::Arc;
use std::time::Duration;
use testcontainers::{core::ContainerPort, runners::AsyncRunner, GenericImage};

#[tokio::test]
async fn distributed_lock() {
    let container = GenericImage::new("redis", "7.2-alpine")
        .with_exposed_port(ContainerPort::Tcp(6379))
        .start()
        .await
        .expect("Failed to start Redis");

    let host_port = container.get_host_port_ipv4(6379).await.expect("Failed to get port");
    let redis_url = format!("redis://127.0.0.1:{}", host_port);

    let repository = Arc::new(
        RedisCacheRepository::new(&redis_url)
            .await
            .expect("Failed to create repository"),
    );
    let lock_key = "resource_lock";
    let lock_value = "unique_lock_id";
    let ttl = Duration::from_secs(5);

    let lock =
        DistributedLock::new(repository.clone(), lock_key.to_string(), lock_value.to_string(), ttl);

    // Acquire lock
    let acquired = lock.acquire().await.expect("Failed to acquire lock");
    assert!(acquired, "Should acquire lock");

    // Try to acquire again (should fail)
    let lock2 =
        DistributedLock::new(repository.clone(), lock_key.to_string(), "other_id".to_string(), ttl);
    let acquired2 = lock2.acquire().await.expect("Failed to checking lock 2");
    assert!(!acquired2, "Should not acquire lock that is already held");

    // Release lock
    lock.release().await.expect("Failed to release lock");

    // Try to acquire again (should succeed)
    let acquired3 = lock2.acquire().await.expect("Failed to acquire lock 3");
    assert!(acquired3, "Should acquire lock after release");
}
