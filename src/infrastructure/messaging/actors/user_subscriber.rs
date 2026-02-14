use ractor::{Actor, ActorProcessingErr, ActorRef};
use async_nats::Client;
use tracing::info;

use crate::infrastructure::messaging::{
    events::{v1::*, v2::*, EventProcessor},
    subjects::{SubjectVersion, UserEventType, UserSubject},
    subscriber,
};


/// User subscriber actor state
pub struct UserSubscriberState {
    client: Client,
    env: String,
}

/// User subscriber actor messages
#[derive(Debug)]
pub enum UserSubscriberMessage {
    /// Subscribe to v1 user events
    SubscribeV1 { event_type: UserEventType },
    /// Subscribe to v2 user events
    SubscribeV2 { event_type: UserEventType },
    /// Subscribe to all v1 events
    SubscribeAllV1,
    /// Subscribe to all v2 events
    SubscribeAllV2,
}

/// User subscriber actor
pub struct UserSubscriberActor;

#[async_trait::async_trait]
impl Actor for UserSubscriberActor {
    type Msg = UserSubscriberMessage;
    type State = UserSubscriberState;
    type Arguments = (Client, String);

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        let (client, env) = args;
        info!("UserSubscriberActor started for environment: {}", env);
        Ok(UserSubscriberState { client, env })
    }

    async fn handle(
        &self,
        _myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            UserSubscriberMessage::SubscribeV1 { event_type } => {
                let subject = UserSubject::build(&state.env, SubjectVersion::V1, event_type);
                info!("Subscribing to v1 subject: {}", subject);
                
                if let Ok(sub) = state.client.subscribe(subject.clone()).await {
                    match event_type {
                        UserEventType::Created => {
                            tokio::spawn(subscriber::handle_subscription(sub, V1CreatedProcessor));
                        }
                        UserEventType::Updated => {
                            tokio::spawn(subscriber::handle_subscription(sub, V1UpdatedProcessor));
                        }
                        UserEventType::Deleted => {
                            tokio::spawn(subscriber::handle_subscription(sub, V1DeletedProcessor));
                        }
                    }
                }
            }
            UserSubscriberMessage::SubscribeV2 { event_type } => {
                let subject = UserSubject::build(&state.env, SubjectVersion::V2, event_type);
                info!("Subscribing to v2 subject: {}", subject);
                
                if let Ok(sub) = state.client.subscribe(subject.clone()).await {
                    match event_type {
                        UserEventType::Created => {
                            tokio::spawn(subscriber::handle_subscription(sub, V2CreatedProcessor));
                        }
                        UserEventType::Updated => {
                            tokio::spawn(subscriber::handle_subscription(sub, V2UpdatedProcessor));
                        }
                        UserEventType::Deleted => {
                            tokio::spawn(subscriber::handle_subscription(sub, V2DeletedProcessor));
                        }
                    }
                }
            }
            UserSubscriberMessage::SubscribeAllV1 => {
                let subject = UserSubject::build_version_wildcard(&state.env, SubjectVersion::V1);
                info!("Subscribing to all v1 events: {}", subject);
                
                if let Ok(sub) = state.client.subscribe(subject.clone()).await {
                    tokio::spawn(subscriber::handle_wildcard_subscription(
                        sub,
                        V1CreatedProcessor,
                        V1UpdatedProcessor,
                        V1DeletedProcessor,
                    ));
                }
            }
            UserSubscriberMessage::SubscribeAllV2 => {
                let subject = UserSubject::build_version_wildcard(&state.env, SubjectVersion::V2);
                info!("Subscribing to all v2 events: {}", subject);
                
                if let Ok(sub) = state.client.subscribe(subject.clone()).await {
                    tokio::spawn(subscriber::handle_wildcard_subscription(
                        sub,
                        V2CreatedProcessor,
                        V2UpdatedProcessor,
                        V2DeletedProcessor,
                    ));
                }
            }
        }
        Ok(())
    }
}

// V1 Event Processors
struct V1CreatedProcessor;
struct V1UpdatedProcessor;
struct V1DeletedProcessor;

#[async_trait::async_trait]
impl EventProcessor for V1CreatedProcessor {
    type Event = UserCreatedEventV1;
    
    async fn process(&self, event: Self::Event) {
        info!("Processing UserCreatedEventV1: user_id={}, email={}", event.user_id, event.email);
        // Add your business logic here
    }
    
    fn event_name(&self) -> &'static str {
        "UserCreatedEventV1"
    }
}

#[async_trait::async_trait]
impl EventProcessor for V1UpdatedProcessor {
    type Event = UserUpdatedEventV1;
    
    async fn process(&self, event: Self::Event) {
        info!("Processing UserUpdatedEventV1: user_id={}", event.user_id);
        // Add your business logic here
    }
    
    fn event_name(&self) -> &'static str {
        "UserUpdatedEventV1"
    }
}

#[async_trait::async_trait]
impl EventProcessor for V1DeletedProcessor {
    type Event = UserDeletedEventV1;
    
    async fn process(&self, event: Self::Event) {
        info!("Processing UserDeletedEventV1: user_id={}", event.user_id);
        // Add your business logic here
    }
    
    fn event_name(&self) -> &'static str {
        "UserDeletedEventV1"
    }
}

// V2 Event Processors
struct V2CreatedProcessor;
struct V2UpdatedProcessor;
struct V2DeletedProcessor;

#[async_trait::async_trait]
impl EventProcessor for V2CreatedProcessor {
    type Event = UserCreatedEventV2;
    
    async fn process(&self, event: Self::Event) {
        info!("Processing UserCreatedEventV2: user_id={}, email={}, role={}", 
              event.user_id, event.email, event.role);
        // Add your business logic here
    }
    
    fn event_name(&self) -> &'static str {
        "UserCreatedEventV2"
    }
}

#[async_trait::async_trait]
impl EventProcessor for V2UpdatedProcessor {
    type Event = UserUpdatedEventV2;
    
    async fn process(&self, event: Self::Event) {
        info!("Processing UserUpdatedEventV2: user_id={}", event.user_id);
        // Add your business logic here
    }
    
    fn event_name(&self) -> &'static str {
        "UserUpdatedEventV2"
    }
}

#[async_trait::async_trait]
impl EventProcessor for V2DeletedProcessor {
    type Event = UserDeletedEventV2;
    
    async fn process(&self, event: Self::Event) {
        info!("Processing UserDeletedEventV2: user_id={}, reason={:?}", 
              event.user_id, event.reason);
        // Add your business logic here
    }
    
    fn event_name(&self) -> &'static str {
        "UserDeletedEventV2"
    }
}

/// Public API to spawn and subscribe
pub async fn spawn_and_subscribe_v1(
    client: Client,
    env: String,
    event_type: UserEventType,
) -> Result<ActorRef<UserSubscriberMessage>, anyhow::Error> {
    let (actor_ref, _) = Actor::spawn(None, UserSubscriberActor, (client, env))
        .await
        .map_err(|e| anyhow::Error::msg(format!("Failed to spawn UserSubscriberActor: {}", e)))?;

    actor_ref
        .cast(UserSubscriberMessage::SubscribeV1 { event_type })
        .map_err(|e| anyhow::Error::msg(format!("Failed to send subscribe message: {}", e)))?;

    Ok(actor_ref)
}

/// Public API to spawn and subscribe to v2
pub async fn spawn_and_subscribe_v2(
    client: Client,
    env: String,
    event_type: UserEventType,
) -> Result<ActorRef<UserSubscriberMessage>, anyhow::Error> {
    let (actor_ref, _) = Actor::spawn(None, UserSubscriberActor, (client, env))
        .await
        .map_err(|e| anyhow::Error::msg(format!("Failed to spawn UserSubscriberActor: {}", e)))?;

    actor_ref
        .cast(UserSubscriberMessage::SubscribeV2 { event_type })
        .map_err(|e| anyhow::Error::msg(format!("Failed to send subscribe message: {}", e)))?;

    Ok(actor_ref)
}

/// Public API to spawn and subscribe to all v1 events
pub async fn spawn_and_subscribe_all_v1(
    client: Client,
    env: String,
) -> Result<ActorRef<UserSubscriberMessage>, anyhow::Error> {
    let (actor_ref, _) = Actor::spawn(None, UserSubscriberActor, (client, env))
        .await
        .map_err(|e| anyhow::Error::msg(format!("Failed to spawn UserSubscriberActor: {}", e)))?;

    actor_ref
        .cast(UserSubscriberMessage::SubscribeAllV1)
        .map_err(|e| anyhow::Error::msg(format!("Failed to send subscribe message: {}", e)))?;

    Ok(actor_ref)
}

/// Public API to spawn and subscribe to all v2 events
pub async fn spawn_and_subscribe_all_v2(
    client: Client,
    env: String,
) -> Result<ActorRef<UserSubscriberMessage>, anyhow::Error> {
    let (actor_ref, _) = Actor::spawn(None, UserSubscriberActor, (client, env))
        .await
        .map_err(|e| anyhow::Error::msg(format!("Failed to spawn UserSubscriberActor: {}", e)))?;

    actor_ref
        .cast(UserSubscriberMessage::SubscribeAllV2)
        .map_err(|e| anyhow::Error::msg(format!("Failed to send subscribe message: {}", e)))?;

    Ok(actor_ref)
}
