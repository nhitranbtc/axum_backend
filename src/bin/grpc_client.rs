use axum_backend::grpc::proto::{user_service_client::UserServiceClient, CreateUserRequest};
use std::env;
use tracing::{info, warn};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing with default INFO level
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // Get configuration from environment or use default
    let grpc_host = env::var("GRPC_HOST").unwrap_or_else(|_| "http://127.0.0.1:50051".to_string());

    info!("🔌 Connecting to gRPC server at {}", grpc_host);

    let mut client = UserServiceClient::connect(grpc_host).await?;
    info!("✅ Connected successfully");

    // Generate unique user data
    let unique_id = Uuid::new_v4();
    let email = format!("grpc_user_{}@example.com", unique_id);
    let name = "gRPC Client User";
    let password = "SecurePassword123!";

    info!("📤 Creating user with email: {}", email);
    
    create_and_print_user(&mut client, &email, name, password).await;

    Ok(())
}

async fn create_and_print_user(
    client: &mut UserServiceClient<tonic::transport::Channel>,
    email: &str,
    name: &str,
    password: &str,
) {
    let request = tonic::Request::new(CreateUserRequest {
        email: email.to_string(),
        name: name.to_string(),
        password: password.to_string(),
        role: None, // Optional field
    });

    match client.create_user(request).await {
        Ok(response) => {
            let user = response.into_inner();
            info!("✅ User Created Successfully:");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("  ID:         {}", user.id);
            println!("  Name:       {}", user.name);
            println!("  Email:      {}", user.email);
            println!("  Created At: {} (Unix timestamp)", user.created_at);
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
        },
        Err(status) => {
            warn!("❌ gRPC Error: {}", status.message());
        }
    }
}
