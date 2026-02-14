use axum_backend::infrastructure::messaging::{
    events::{v2::*, traits::Event},
    subjects::{SubjectVersion, UserSubject},
};
use futures::StreamExt;
use std::env;
use tokio::signal;
use tracing::{error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Configuration for the NATS subscriber
struct Config {
    nats_url: String,
    nats_user: Option<String>,
    nats_password: Option<String>,
    nats_token: Option<String>,
    app_env: String,
}

impl Config {
    fn from_env() -> Self {
        Self {
            nats_url: env::var("NATS_URL")
                .unwrap_or_else(|_| "nats://localhost:4222".to_string()),
            nats_user: env::var("NATS_USER").ok(),
            nats_password: env::var("NATS_PASSWORD").ok(),
            nats_token: env::var("NATS_TOKEN").ok(),
            app_env: env::var("APP_ENV").unwrap_or_else(|_| "dev".to_string()),
        }
    }
}

/// Process incoming NATS messages and deserialize them into typed events
async fn process_message(message: async_nats::Message) {
    let subject = message.subject.as_str();
    
    info!(
        subject = %subject,
        payload_size = message.payload.len(),
        "📨 Received message"
    );

    // Determine event type from subject
    if subject.contains(".created") {
        match UserCreatedEventV2::from_bytes(&message.payload) {
            Ok(event) => {
                info!(
                    user_id = %event.user_id,
                    email = %event.email,
                    name = %event.name,
                    "✅ UserCreated event processed"
                );
                // Here you could trigger additional actions:
                // - Send welcome email
                // - Update analytics
                // - Sync to external systems
            }
            Err(e) => {
                error!(
                    subject = %subject,
                    error = %e,
                    "❌ Failed to deserialize UserCreatedEventV2"
                );
            }
        }
    } else if subject.contains(".updated") {
        match UserUpdatedEventV2::from_bytes(&message.payload) {
            Ok(event) => {
                let mut changes = Vec::new();
                if let Some(name_change) = &event.name_change {
                    changes.push(format!(
                        "name: {:?} → {:?}",
                        name_change.previous_value, name_change.new_value
                    ));
                }
                if let Some(email_change) = &event.email_change {
                    changes.push(format!(
                        "email: {:?} → {:?}",
                        email_change.previous_value, email_change.new_value
                    ));
                }
                
                info!(
                    user_id = %event.user_id,
                    changes = %changes.join(", "),
                    updated_by = ?event.updated_by,
                    "✅ UserUpdated event processed"
                );
                // Here you could trigger additional actions:
                // - Invalidate caches
                // - Update search indexes
                // - Audit logging
            }
            Err(e) => {
                error!(
                    subject = %subject,
                    error = %e,
                    "❌ Failed to deserialize UserUpdatedEventV2"
                );
            }
        }
    } else if subject.contains(".deleted") {
        match UserDeletedEventV2::from_bytes(&message.payload) {
            Ok(event) => {
                info!(
                    user_id = %event.user_id,
                    deleted_by = ?event.deleted_by,
                    reason = ?event.reason,
                    "✅ UserDeleted event processed"
                );
                // Here you could trigger additional actions:
                // - Clean up user data
                // - Revoke sessions
                // - Archive records
            }
            Err(e) => {
                error!(
                    subject = %subject,
                    error = %e,
                    "❌ Failed to deserialize UserDeletedEventV2"
                );
            }
        }
    } else {
        warn!(
            subject = %subject,
            "⚠️  Unknown event type, skipping deserialization"
        );
        // Log raw payload for debugging
        if let Ok(payload_str) = String::from_utf8(message.payload.to_vec()) {
            tracing::debug!(payload = %payload_str, "Raw payload");
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // 1. Initialize logging with better formatting
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(false)
                .with_thread_ids(false)
        )
        .init();

    // 2. Load configuration
    let config = Config::from_env();
    
    info!("🚀 Starting NATS Event Subscriber");
    info!("Environment: {}", config.app_env);
    info!("NATS URL: {}", config.nats_url);

    // 3. Configure connection options
    let mut options = async_nats::ConnectOptions::new()
        .name("axum-backend-subscriber"); // Set client name for identification
    
    if let (Some(user), Some(pass)) = (&config.nats_user, &config.nats_password) {
        info!("🔐 Using username/password authentication");
        options = options.user_and_password(user.clone(), pass.clone());
    } else if let Some(token) = &config.nats_token {
        info!("🔐 Using token authentication");
        options = options.token(token.clone());
    } else {
        warn!("⚠️  No authentication credentials provided");
    }

    // 4. Connect to NATS
    let client = options.connect(&config.nats_url).await.map_err(|e| {
        error!("❌ Failed to connect to NATS: {}", e);
        e
    })?;
    
    info!("✅ Connected to NATS successfully");

    // 5. Subscribe to all v2 user events
    let subject = UserSubject::build_version_wildcard(&config.app_env, SubjectVersion::V2);
    
    let mut subscriber = client.subscribe(subject.clone()).await.map_err(|e| {
        error!("❌ Failed to subscribe to '{}': {}", subject, e);
        e
    })?;
    
    info!("📡 Subscribed to: {}", subject);
    info!("⏳ Waiting for messages... (Press Ctrl+C to stop)");

    // 6. Process messages with graceful shutdown
    loop {
        tokio::select! {
            Some(message) = subscriber.next() => {
                // Spawn a task for concurrent processing
                tokio::spawn(async move {
                    process_message(message).await;
                });
            }
            _ = signal::ctrl_c() => {
                info!("🛑 Received Ctrl+C, shutting down...");
                break;
            }
        }
    }

    info!("👋 Subscriber stopped");
    Ok(())
}
