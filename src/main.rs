use axum_backend::{
    config::AppConfig,
    infrastructure::database::{connection::create_pool, connection::run_migrations},
    presentation::routes::create_router,
    shared::init_telemetry,
};
use axum_prometheus::PrometheusMetricLayer;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize telemetry (logging)
    init_telemetry();

    // Load configuration
    let config = AppConfig::from_env()?;
    tracing::info!("Configuration loaded successfully");

    // Create database connection pool
    let pool = create_pool(&config.db_config, &config.database_url).await?;
    tracing::info!("Database connection pool created");

    // Run migrations
    run_migrations(&config.database_url).await?;
    tracing::info!("Database migrations completed");

    // Create monitoring layer
    let (prometheus_layer, metric_handle) = PrometheusMetricLayer::pair();

    // Create Email Service
    let email_service = std::sync::Arc::new(
        axum_backend::infrastructure::email::lettre_service::LettreEmailService::new()
            .expect("Failed to create email service"),
    );

    // Create application router
    let app = create_router(
        pool,
        config.jwt_secret.clone(),
        config.jwt_access_expiry,
        config.jwt_refresh_expiry,
        config.jwt_issuer.clone(),
        config.jwt_audience.clone(),
        config.confirm_code_expiry,
        prometheus_layer,
        metric_handle,
        email_service,
    );

    // Parse server address
    let addr: SocketAddr = config.server_address().parse()?;
    tracing::info!("Starting server on {}", addr);

    // Start server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("Server listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
