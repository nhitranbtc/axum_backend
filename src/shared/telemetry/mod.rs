use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialize telemetry (logging and tracing)
pub fn init_telemetry() {
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,axum_backend=debug")),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Telemetry initialized");
}
