pub mod error;
pub mod connection;
pub mod nats_client;
pub mod subscriber;
// pub mod event_publisher; // Moved to producer
// pub mod event_envelope; // Moved to types
pub mod actors;
pub mod subjects;
pub mod events;



pub use error::{MessagingError, Result};
pub use nats_client::NatsClient;
pub use subjects::{SubjectVersion, UserEventType, UserSubject};


#[async_trait::async_trait]
pub trait MessagingService: Send + Sync {
    /// Publish a message to a subject
    async fn publish(&self, subject: String, payload: bytes::Bytes) -> Result<()>;

    /// Subscribe to a subject
    async fn subscribe(&self, subject: String) -> Result<async_nats::Subscriber>;
}

