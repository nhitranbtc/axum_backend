use axum_backend::{
    config::AppConfig,
    grpc::{
        presentation::services::user_service::UserServiceImpl,
        proto::user_service_server::UserServiceServer,
    },
    infrastructure::database::scylla::connection::create_scylla_session,
    shared::init_telemetry,
};
use tonic::transport::Server;
use tonic_web::GrpcWebLayer;
use tower_http::cors::{AllowOrigin, CorsLayer};

// Include the file descriptor set for reflection
const FILE_DESCRIPTOR_SET: &[u8] = include_bytes!("../../target/descriptor.bin");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize telemetry (logging)
    init_telemetry();

    // Load configuration from environment
    let config = AppConfig::from_env()?;
    tracing::info!("Configuration loaded successfully");

    // Create ScyllaDB session (initialises keyspace + tables automatically)
    let scylla_session = std::sync::Arc::new(create_scylla_session(&config.scylla).await?);
    tracing::info!("ScyllaDB session created and schema initialized");

    let addr = format!("0.0.0.0:{}", config.grpc_port).parse()?;

    // Create user service with actor pool
    // Use configured actor pool size (default: 20)
    let actor_pool_size = config.grpc_actor_pool_size;
    let user_service = UserServiceImpl::new(scylla_session, actor_pool_size).await?;

    tracing::info!("🚀 gRPC Server starting on {}", addr);
    tracing::info!("📋 Configuration:");
    tracing::info!("   - Port: {}", config.grpc_port);
    tracing::info!("   - Reflection: {}", config.grpc_reflection_enabled);
    tracing::info!("   - gRPC-Web: {}", config.grpc_web_enabled);
    tracing::info!("   - Max Connections: {}", config.grpc_max_connections);
    tracing::info!("   - Actor Pool Size: {}", user_service.pool_size());
    tracing::info!("   - CORS Origins: {:?}", config.grpc_cors_origins);
    tracing::info!("   - Database: ScyllaDB Connected ✅");

    // Configure CORS based on allowed origins
    let cors = if config.grpc_cors_origins.contains(&"*".to_string()) {
        tracing::warn!("⚠️  CORS configured with wildcard (*) - not recommended for production!");
        CorsLayer::permissive()
    } else {
        let origins: Vec<_> = config
            .grpc_cors_origins
            .iter()
            .filter_map(|origin| origin.parse().ok())
            .collect();

        if !origins.is_empty() {
            tracing::info!("✅ CORS configured with specific origins");
            CorsLayer::new()
                .allow_origin(AllowOrigin::list(origins))
                .allow_methods(tower_http::cors::Any)
                .allow_headers(tower_http::cors::Any)
                .expose_headers(tower_http::cors::Any)
        } else {
            tracing::warn!("⚠️  No valid CORS origins, using permissive CORS");
            CorsLayer::permissive()
        }
    };

    if config.grpc_web_enabled {
        tracing::info!("🌐 gRPC-Web enabled - browsers can connect directly!");
    }

    // Build server with all layers
    let mut server_builder = Server::builder()
        .accept_http1(config.grpc_web_enabled)
        .layer(cors)
        .layer(GrpcWebLayer::new());

    // Add user service
    let mut router = server_builder.add_service(UserServiceServer::new(user_service.clone()));

    // Add reflection service if enabled
    if config.grpc_reflection_enabled {
        let reflection_service = tonic_reflection::server::Builder::configure()
            .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
            .build_v1()?;
        router = router.add_service(reflection_service);
        tracing::info!("📡 Server reflection enabled - use grpcurl to discover services");
    }

    tracing::info!("✅ Server initialized, starting to serve requests...");

    // Create a future for the server
    let server_future = router.serve(addr);

    // Run server with graceful shutdown
    tokio::select! {
        result = server_future => {
            result?;
        }
        _ = shutdown_signal() => {
            tracing::info!("🔄 Graceful shutdown initiated");
            // Shutdown actor pool
            user_service.shutdown().await;
        }
    }

    tracing::info!("👋 Server shutdown complete");
    Ok(())
}

/// Handles graceful shutdown on SIGTERM or SIGINT (Ctrl+C)
async fn shutdown_signal() {
    use tokio::signal;

    let ctrl_c = async {
        signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("🛑 Received Ctrl+C signal");
        },
        _ = terminate => {
            tracing::info!("🛑 Received termination signal");
        },
    }
}
