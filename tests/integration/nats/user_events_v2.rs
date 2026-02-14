use axum_backend::infrastructure::messaging::{
    MessagingService, NatsClient,
    events::{v2::UserCreatedEventV2, Event},
    subjects::{SubjectVersion, UserEventType, UserSubject},
};

use std::time::Duration;
use testcontainers::{core::ContainerPort, runners::AsyncRunner, GenericImage, ImageExt};
use tokio::time::timeout;
use futures::StreamExt;

#[tokio::test]
async fn test_user_events_v2_created_subscription() {
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

    // Build subject for v2 created events
    let subject = UserSubject::build("test", SubjectVersion::V2, UserEventType::Created);
    
    // Subscribe
    let mut subscriber = client.subscribe(subject.clone()).await.expect("Failed to subscribe");

    // Create and publish event with v2 enhanced fields
    let event = UserCreatedEventV2::new(
        "user-789".to_string(),
        "v2test@example.com".to_string(),
        "Alice Smith".to_string(),
        "admin".to_string(),
        true,
        true,
    )
    .with_metadata("source".to_string(), "api".to_string())
    .with_metadata("ip_address".to_string(), "192.168.1.1".to_string());
    
    let payload = event.to_bytes().expect("Failed to serialize event");
    client.publish(subject.clone(), payload).await.expect("Failed to publish");

    // Receive and verify
    let message = timeout(Duration::from_secs(5), subscriber.next())
        .await
        .expect("Timed out waiting for message")
        .expect("Stream ended unexpectedly");

    assert_eq!(message.subject.to_string(), subject);
    
    let received_event = UserCreatedEventV2::from_bytes(&message.payload)
        .expect("Failed to deserialize event");
    
    assert_eq!(received_event.user_id, "user-789");
    assert_eq!(received_event.email, "v2test@example.com");
    assert_eq!(received_event.name, "Alice Smith");
    assert_eq!(received_event.role, "admin");
    assert_eq!(received_event.is_active, true);
    assert_eq!(received_event.is_email_verified, true);
    assert_eq!(received_event.metadata.get("source"), Some(&"api".to_string()));
    assert_eq!(received_event.metadata.get("ip_address"), Some(&"192.168.1.1".to_string()));
}

#[tokio::test]
async fn test_user_events_v2_wildcard_subscription() {
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

    // Subscribe to all v2 events using wildcard
    let wildcard_subject = UserSubject::build_version_wildcard("test", SubjectVersion::V2);
    let mut subscriber = client.subscribe(wildcard_subject).await.expect("Failed to subscribe");

    // Publish to specific subject
    let subject = UserSubject::build("test", SubjectVersion::V2, UserEventType::Created);
    let event = UserCreatedEventV2::new(
        "user-999".to_string(),
        "wildcard-v2@example.com".to_string(),
        "Bob Johnson".to_string(),
        "user".to_string(),
        false,
        false,
    );
    
    let payload = event.to_bytes().expect("Failed to serialize event");
    client.publish(subject, payload).await.expect("Failed to publish");

    // Receive and verify
    let message = timeout(Duration::from_secs(5), subscriber.next())
        .await
        .expect("Timed out waiting for message")
        .expect("Stream ended unexpectedly");

    let received_event = UserCreatedEventV2::from_bytes(&message.payload)
        .expect("Failed to deserialize event");
    
    assert_eq!(received_event.user_id, "user-999");
    assert_eq!(received_event.email, "wildcard-v2@example.com");
    assert_eq!(received_event.role, "user");
}
