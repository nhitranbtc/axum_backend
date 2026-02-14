use axum_backend::infrastructure::messaging::{MessagingService, NatsClient};
use bytes::Bytes;
use std::time::Duration;
use testcontainers::{core::ContainerPort, runners::AsyncRunner, GenericImage, ImageExt};
use tokio::time::timeout;
use futures::StreamExt;

#[tokio::test]
async fn test_nats_pub_sub() {
    // Start NATS container
    let container = GenericImage::new("nats", "latest")
        .with_exposed_port(ContainerPort::Tcp(4222))
        .with_cmd(vec!["-js"])
        .start()
        .await
        .expect("Failed to start NATS container");

    // Give NATS a moment to be fully ready
    tokio::time::sleep(Duration::from_millis(500)).await;

    let port = container.get_host_port_ipv4(4222).await.expect("Failed to get NATS port");
    let nats_url = format!("nats://127.0.0.1:{}", port);

    // Ensure we don't accidentally use auth from env if present
    unsafe {
        std::env::remove_var("NATS_USER");
        std::env::remove_var("NATS_PASSWORD");
        std::env::remove_var("NATS_TOKEN");
    }

    // Creates client
    let client = NatsClient::new(&nats_url).await.expect("Failed to connect to NATS");

    let subject = "prod.users.connectivity";
    let payload = Bytes::from("Hello, NATS!");

    // Subscribe
    let mut subscriber = client.subscribe(subject.to_string()).await.expect("Failed to subscribe");

    // Publish
    client.publish(subject.to_string(), payload.clone()).await.expect("Failed to publish");

    // Receive
    let message = timeout(Duration::from_secs(5), subscriber.next())
        .await
        .expect("Timed out waiting for message")
        .expect("Stream ended unexpectedly");

    assert_eq!(message.subject.to_string(), subject);
    assert_eq!(message.payload, payload);
}
