use super::{MessagingService, connection, Result};
use async_nats::{Client, Subscriber};
use bytes::Bytes;

#[derive(Clone)]
pub struct NatsClient {
    client: Client,
}

impl NatsClient {
    pub async fn new(url: &str) -> Result<Self> {
        let client = connection::connect(url).await?;
        Ok(Self { client })
    }
    
    /// Get a reference to the underlying NATS client
    pub fn client(&self) -> &Client {
        &self.client
    }
}

#[async_trait::async_trait]
impl MessagingService for NatsClient {
    async fn publish(&self, subject: String, payload: Bytes) -> Result<()> {
        connection::publish(&self.client, subject, payload).await
    }

    async fn subscribe(&self, subject: String) -> Result<Subscriber> {
        connection::subscribe(&self.client, subject).await
    }
}
