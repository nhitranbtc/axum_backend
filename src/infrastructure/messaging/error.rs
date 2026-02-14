use thiserror::Error;

/// Custom error types for messaging infrastructure
#[derive(Debug, Error)]
pub enum MessagingError {
    #[error("Failed to connect to NATS at {url}: {source}")]
    ConnectionFailed {
        url: String,
        #[source]
        source: async_nats::ConnectError,
    },

    #[error("Failed to publish to subject '{subject}': {source}")]
    PublishFailed {
        subject: String,
        #[source]
        source: async_nats::PublishError,
    },

    #[error("Failed to subscribe to subject '{subject}': {source}")]
    SubscribeFailed {
        subject: String,
        #[source]
        source: async_nats::SubscribeError,
    },

    #[error("Failed to serialize event: {0}")]
    SerializationFailed(#[from] serde_json::Error),

    #[error("Actor spawn failed: {0}")]
    ActorSpawnFailed(String),

    #[error("Failed to send message to actor: {0}")]
    ActorMessageFailed(String),
}

/// Result type alias for messaging operations
pub type Result<T> = std::result::Result<T, MessagingError>;

/// Convert from anyhow::Error for backward compatibility during migration
impl From<anyhow::Error> for MessagingError {
    fn from(err: anyhow::Error) -> Self {
        MessagingError::ActorSpawnFailed(err.to_string())
    }
}
