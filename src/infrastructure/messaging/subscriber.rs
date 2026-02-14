use async_nats::Subscriber;
use futures::StreamExt;
use tracing::{error, info};

use crate::infrastructure::messaging::events::{Event, EventProcessor};

/// Generic subscription handler that processes events using an EventProcessor
pub async fn handle_subscription<P>(mut subscriber: Subscriber, processor: P)
where
    P: EventProcessor,
{
    info!("Starting subscription handler for {}", processor.event_name());
    
    while let Some(message) = subscriber.next().await {
        match P::Event::from_bytes(&message.payload) {
            Ok(event) => {
                processor.process(event).await;
            }
            Err(e) => {
                error!(
                    "Failed to deserialize {} from subject {}: {:?}",
                    processor.event_name(),
                    message.subject,
                    e
                );
            }
        }
    }
    
    info!("Subscription handler for {} ended", processor.event_name());
}

/// Generic wildcard subscription handler that routes to appropriate processors
pub async fn handle_wildcard_subscription<P1, P2, P3>(
    mut subscriber: Subscriber,
    created_processor: P1,
    updated_processor: P2,
    deleted_processor: P3,
)
where
    P1: EventProcessor,
    P2: EventProcessor,
    P3: EventProcessor,
{
    info!("Starting wildcard subscription handler");
    
    while let Some(message) = subscriber.next().await {
        let subject = message.subject.to_string();
        
        if subject.ends_with(".created") {
            if let Ok(event) = P1::Event::from_bytes(&message.payload) {
                created_processor.process(event).await;
            } else {
                error!("Failed to deserialize created event from {}", subject);
            }
        } else if subject.ends_with(".updated") {
            if let Ok(event) = P2::Event::from_bytes(&message.payload) {
                updated_processor.process(event).await;
            } else {
                error!("Failed to deserialize updated event from {}", subject);
            }
        } else if subject.ends_with(".deleted") {
            if let Ok(event) = P3::Event::from_bytes(&message.payload) {
                deleted_processor.process(event).await;
            } else {
                error!("Failed to deserialize deleted event from {}", subject);
            }
        } else {
            error!("Unknown event type from subject: {}", subject);
        }
    }
    
    info!("Wildcard subscription handler ended");
}
