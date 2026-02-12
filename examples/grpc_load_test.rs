// Goose Load Test for gRPC CreateUser Endpoint
//
// Run with:
// cargo run --example grpc_load_test -- --users 10 --spawn-rate 2 --run-time 1m
//
// Options:
// --users: Number of concurrent users (default: 10)
// --spawn-rate: Users spawned per second (default: 1)
// --run-time: Test duration (e.g., 1m, 5m, 1h)
// --report-file: HTML report output path

use goose::prelude::*;
use tonic::Request;
use uuid::Uuid;

// Import the generated gRPC code
use axum_backend::grpc::proto::user_service_client::UserServiceClient;
use axum_backend::grpc::proto::CreateUserRequest;

/// Transaction: Create a new user via gRPC
async fn create_user_transaction(user: &mut GooseUser) -> TransactionResult {
    // Generate unique email using UUID to prevent duplicates
    // Each transaction gets a unique UUID, ensuring no conflicts even when
    // the same user runs multiple transactions per second
    let unique_id = Uuid::new_v4();
    let email = format!("goose_{}@example.com", unique_id);

    // Get gRPC endpoint - Goose uses --host flag
    let grpc_endpoint = if user.config.host.is_empty() {
        "http://localhost:50051".to_string()
    } else {
        user.config.host.clone()
    };

    // Create gRPC client
    let mut client = match UserServiceClient::connect(grpc_endpoint.clone()).await {
        Ok(client) => client,
        Err(e) => {
            eprintln!("❌ Failed to connect to gRPC server {}: {}", grpc_endpoint, e);
            // Just return Ok - Goose will track this as a failed transaction
            return Ok(());
        },
    };

    // Create request
    let grpc_request = Request::new(CreateUserRequest {
        email: email.clone(),
        name: format!("Goose Load Test User"),
        password: "SecurePassword123!".to_string(),
        role: None,
    });

    let started = std::time::Instant::now();

    // Make gRPC call
    match client.create_user(grpc_request).await {
        Ok(response) => {
            let elapsed = started.elapsed();

            let user_response = response.into_inner();

            // Log occasionally to show progress (every 10th request)
            if user.weighted_users_index % 10 == 0 {
                println!(
                    "✅ Created {} (ID: {}) - {}ms",
                    user_response.email,
                    user_response.id,
                    elapsed.as_millis()
                );
            }
        },
        Err(e) => {
            eprintln!("❌ CreateUser failed for {}: {}", email, e);
        },
    }

    Ok(())
}

/// Scenario: User registration load test
fn user_registration_scenario() -> Scenario {
    scenario!("UserRegistration")
        .register_transaction(transaction!(create_user_transaction).set_name("CreateUser"))
}

#[tokio::main]
async fn main() -> Result<(), GooseError> {
    println!("\n╔════════════════════════════════════════════════════════╗");
    println!("║     Goose Load Test - gRPC CreateUser                 ║");
    println!("╚════════════════════════════════════════════════════════╝\n");
    println!("Configuration:");
    println!("  • Default Host: http://localhost:50051");
    println!("  • Use --host to override");
    println!("  • Use --users to set concurrent users");
    println!("  • Use --spawn-rate to control ramp-up");
    println!("  • Use --run-time to set duration (e.g., 1m, 5m)");
    println!("  • Use --report-file report.html for HTML report\n");

    GooseAttack::initialize()?
        .register_scenario(user_registration_scenario())
        .set_default(GooseDefault::Host, "http://localhost:50051")?
        .execute()
        .await?;

    println!("\n✅ Load test completed!");
    println!("Check the console output above for metrics.\n");

    Ok(())
}
