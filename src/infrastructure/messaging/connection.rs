use async_nats::{Client, ConnectOptions};
use tracing::{info, error};
use std::env;

use super::error::{MessagingError, Result};

/// Connect to NATS server with authentication from environment variables
pub async fn connect(url: &str) -> Result<Client> {
    info!("Connecting to NATS at {}", url);
    
    let mut options = ConnectOptions::new();
    
    // Configure authentication from environment variables
    if let (Ok(user), Ok(pass)) = (env::var("NATS_USER"), env::var("NATS_PASSWORD")) {
        info!("Using username/password authentication");
        options = options.user_and_password(user, pass);
    } else if let Ok(token) = env::var("NATS_TOKEN") {
        info!("Using token authentication");
        options = options.token(token);
    }
    
    options.connect(url).await
        .map_err(|e| {
            error!("Failed to connect to NATS at {}: {}", url, e);
            MessagingError::ConnectionFailed {
                url: url.to_string(),
                source: e,
            }
        })
}

/// Publish a message to NATS (Fire and Forget)
pub async fn publish(client: &Client, subject: String, payload: bytes::Bytes) -> Result<()> {
    client
        .publish(subject.clone(), payload)
        .await
        .map_err(|e| {
            error!("Failed to publish to {}: {}", subject, e);
            MessagingError::PublishFailed {
                subject,
                source: e,
            }
        })?;
    Ok(())
}

/// Subscribe to a NATS subject
pub async fn subscribe(client: &Client, subject: String) -> Result<async_nats::Subscriber> {
    client
        .subscribe(subject.clone())
        .await
        .map_err(|e| {
            error!("Failed to subscribe to {}: {}", subject, e);
            MessagingError::SubscribeFailed {
                subject,
                source: e,
            }
        })
}
