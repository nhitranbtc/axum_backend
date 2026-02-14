use axum_backend::infrastructure::messaging::{
    MessagingService, NatsClient,
    events::{v1::UserCreatedEventV1, v2::UserCreatedEventV2, Event},
    subjects::{SubjectVersion, UserEventType, UserSubject},
};
use std::time::Duration;
use testcontainers::{core::ContainerPort, runners::AsyncRunner, GenericImage, ImageExt};
use tokio::time::timeout;
use futures::StreamExt;

#[tokio::test]
async fn test_concurrent_version_subscriptions() {
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

    // Subscribe to v1 events
    let v1_subject = UserSubject::build("test", SubjectVersion::V1, UserEventType::Created);
    let mut v1_subscriber = client.subscribe(v1_subject.clone()).await.expect("Failed to subscribe to v1");

    // Subscribe to v2 events
    let v2_subject = UserSubject::build("test", SubjectVersion::V2, UserEventType::Created);
    let mut v2_subscriber = client.subscribe(v2_subject.clone()).await.expect("Failed to subscribe to v2");

    // Publish v1 event
    let v1_event = UserCreatedEventV1::new(
        "user-v1-001".to_string(),
        "v1@example.com".to_string(),
        "V1 User".to_string(),
    );
    let v1_payload = v1_event.to_bytes().expect("Failed to serialize v1 event");
    client.publish(v1_subject, v1_payload).await.expect("Failed to publish v1");

    // Publish v2 event
    let v2_event = UserCreatedEventV2::new(
        "user-v2-001".to_string(),
        "v2@example.com".to_string(),
        "V2 User".to_string(),
        "admin".to_string(),
        true,
        true,
    );
    let v2_payload = v2_event.to_bytes().expect("Failed to serialize v2 event");
    client.publish(v2_subject, v2_payload).await.expect("Failed to publish v2");

    // Receive v1 event
    let v1_message = timeout(Duration::from_secs(5), v1_subscriber.next())
        .await
        .expect("Timed out waiting for v1 message")
        .expect("V1 stream ended unexpectedly");

    let received_v1 = UserCreatedEventV1::from_bytes(&v1_message.payload)
        .expect("Failed to deserialize v1 event");
    
    assert_eq!(received_v1.user_id, "user-v1-001");
    assert_eq!(received_v1.email, "v1@example.com");

    // Receive v2 event
    let v2_message = timeout(Duration::from_secs(5), v2_subscriber.next())
        .await
        .expect("Timed out waiting for v2 message")
        .expect("V2 stream ended unexpectedly");

    let received_v2 = UserCreatedEventV2::from_bytes(&v2_message.payload)
        .expect("Failed to deserialize v2 event");
    
    assert_eq!(received_v2.user_id, "user-v2-001");
    assert_eq!(received_v2.email, "v2@example.com");
    assert_eq!(received_v2.role, "admin");
}

#[tokio::test]
async fn test_no_cross_version_event_leakage() {
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

    // Subscribe ONLY to v1 events
    let v1_subject = UserSubject::build("test", SubjectVersion::V1, UserEventType::Created);
    let mut v1_subscriber = client.subscribe(v1_subject.clone()).await.expect("Failed to subscribe to v1");

    // Publish v2 event (should NOT be received by v1 subscriber)
    let v2_subject = UserSubject::build("test", SubjectVersion::V2, UserEventType::Created);
    let v2_event = UserCreatedEventV2::new(
        "user-v2-leak".to_string(),
        "leak@example.com".to_string(),
        "Leak Test".to_string(),
        "user".to_string(),
        true,
        false,
    );
    let v2_payload = v2_event.to_bytes().expect("Failed to serialize v2 event");
    client.publish(v2_subject, v2_payload).await.expect("Failed to publish v2");

    // Publish v1 event (should be received)
    let v1_event = UserCreatedEventV1::new(
        "user-v1-valid".to_string(),
        "valid@example.com".to_string(),
        "Valid User".to_string(),
    );
    let v1_payload = v1_event.to_bytes().expect("Failed to serialize v1 event");
    client.publish(v1_subject, v1_payload).await.expect("Failed to publish v1");

    // Should receive only the v1 event
    let message = timeout(Duration::from_secs(2), v1_subscriber.next())
        .await
        .expect("Timed out waiting for message")
        .expect("Stream ended unexpectedly");

    let received = UserCreatedEventV1::from_bytes(&message.payload)
        .expect("Failed to deserialize event");
    
    // Verify we received the v1 event, not the v2 event
    assert_eq!(received.user_id, "user-v1-valid");
    assert_eq!(received.email, "valid@example.com");

    // Verify no more messages are received (v2 event should not leak)
    let no_more_messages = timeout(Duration::from_millis(500), v1_subscriber.next()).await;
    assert!(no_more_messages.is_err(), "Should not receive any more messages");
}
