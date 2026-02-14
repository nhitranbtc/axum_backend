use axum_backend::infrastructure::messaging::{
    MessagingService, NatsClient,
    events::{v1::UserCreatedEventV1, Event},
    subjects::{SubjectVersion, UserEventType, UserSubject},
};

use std::time::Duration;
use testcontainers::{core::ContainerPort, runners::AsyncRunner, GenericImage, ImageExt};
use tokio::time::timeout;
use futures::StreamExt;

#[tokio::test]
async fn test_user_events_v1_created_subscription() {
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

    // Create client
    let client = NatsClient::new(&nats_url).await.expect("Failed to connect to NATS");

    // Build subject for v1 created events
    let subject = UserSubject::build("test", SubjectVersion::V1, UserEventType::Created);
    
    // Subscribe
    let mut subscriber = client.subscribe(subject.clone()).await.expect("Failed to subscribe");

    // Create and publish event
    let event = UserCreatedEventV1::new(
        "user-123".to_string(),
        "test@example.com".to_string(),
        "John Doe".to_string(),
    );
    
    let payload = event.to_bytes().expect("Failed to serialize event");
    client.publish(subject.clone(), payload).await.expect("Failed to publish");

    // Receive and verify
    let message = timeout(Duration::from_secs(5), subscriber.next())
        .await
        .expect("Timed out waiting for message")
        .expect("Stream ended unexpectedly");

    assert_eq!(message.subject.to_string(), subject);
    
    let received_event = UserCreatedEventV1::from_bytes(&message.payload)
        .expect("Failed to deserialize event");
    
    assert_eq!(received_event.user_id, "user-123");
    assert_eq!(received_event.email, "test@example.com");
    assert_eq!(received_event.name, "John Doe");
}

#[tokio::test]
async fn test_user_events_v1_wildcard_subscription() {
    // Start NATS container
    let container = GenericImage::new("nats", "latest")
        .with_exposed_port(ContainerPort::Tcp(4222))
        .with_cmd(vec!["-js"])
        .start()
        .await
        .expect("Failed to start NATS container");

    tokio::time::sleep(Duration::from_millis(500)).await;

    let port = container.get_host_port_ipv4(4222).await.expect("Failed to get NATS port");
    let nats_url = format!("nats://127.0.0.1:{}", port);

    unsafe {
        std::env::remove_var("NATS_USER");
        std::env::remove_var("NATS_PASSWORD");
        std::env::remove_var("NATS_TOKEN");
    }

    let client = NatsClient::new(&nats_url).await.expect("Failed to connect to NATS");

    // Subscribe to all v1 events using wildcard
    let wildcard_subject = UserSubject::build_version_wildcard("test", SubjectVersion::V1);
    let mut subscriber = client.subscribe(wildcard_subject).await.expect("Failed to subscribe");

    // Publish to specific subject
    let subject = UserSubject::build("test", SubjectVersion::V1, UserEventType::Created);
    let event = UserCreatedEventV1::new(
        "user-456".to_string(),
        "wildcard@example.com".to_string(),
        "Jane Doe".to_string(),
    );
    
    let payload = event.to_bytes().expect("Failed to serialize event");
    client.publish(subject, payload).await.expect("Failed to publish");

    // Receive and verify
    let message = timeout(Duration::from_secs(5), subscriber.next())
        .await
        .expect("Timed out waiting for message")
        .expect("Stream ended unexpectedly");

    let received_event = UserCreatedEventV1::from_bytes(&message.payload)
        .expect("Failed to deserialize event");
    
    assert_eq!(received_event.user_id, "user-456");
    assert_eq!(received_event.email, "wildcard@example.com");
}
