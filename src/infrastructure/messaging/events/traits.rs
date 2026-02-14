use bytes::Bytes;
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::infrastructure::messaging::error::{MessagingError, Result};


/// Trait for all event types that can be serialized and sent via messaging
pub trait Event: Serialize + DeserializeOwned + Send + Sync {
    /// Serialize event to bytes for transmission
    fn to_bytes(&self) -> Result<Bytes> {
        let json = serde_json::to_vec(self)?;
        Ok(Bytes::from(json))
    }
    
    /// Deserialize event from bytes
    fn from_bytes(data: &[u8]) -> Result<Self> {
        Ok(serde_json::from_slice(data)?)
    }
}

/// Trait for processing events asynchronously
#[async_trait::async_trait]
pub trait EventProcessor: Send + Sync {
    type Event: Event;
    
    /// Process a received event
    async fn process(&self, event: Self::Event);
    
    /// Get the name of this event type for logging
    fn event_name(&self) -> &'static str;
}
