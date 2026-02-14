use chrono::Utc;
use std::env;
use tracing::{error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // 1. Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 2. Load configuration from env
    let nats_url = env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());
    let nats_user = env::var("NATS_USER").ok();
    let nats_password = env::var("NATS_PASSWORD").ok();
    let nats_token = env::var("NATS_TOKEN").ok();

    info!("Connecting to NATS at {}...", nats_url);

    // 3. Configure connection options
    let mut options = async_nats::ConnectOptions::new();
    
    if let (Some(user), Some(pass)) = (nats_user, nats_password) {
        info!("Using username/password authentication");
        options = options.user_and_password(user, pass);
    } else if let Some(token) = nats_token {
        info!("Using token authentication");
        options = options.token(token);
    } else {
        warn!("No authentication credentials provided");
    }

    // 4. Connect
    let client = match options.connect(&nats_url).await {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to connect to NATS: {}", e);
            return Err(e.into());
        }
    };
    info!("Connected successfully!");

    // 5. Publish a simple message (no domain events)
    let subject = "test.subject";
    let payload = "Hello NATS!";

    info!("Publishing message to {}: {}", subject, payload);
    match client.publish(subject.to_string(), payload.into()).await {
        Ok(_) => info!("Message published successfully"),
        Err(e) => error!("Failed to publish message: {}", e),
    }

    Ok(())
}
