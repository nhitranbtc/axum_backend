use crate::common::server::TestServer;
use axum_backend::{
    application::services::email::{EmailService, EmailType, Recipient},
    infrastructure::email::lettre_service::LettreEmailService,
};
use std::sync::Arc;

/// Test direct email service functionality (Unit-like integration)
#[tokio::test]
#[ignore]
async fn test_service_send_welcome_email() {
    // 1. Load environment variables
    dotenvy::dotenv().ok();

    if std::env::var("SMTP_PASSWORD").is_err() && std::env::var("SMTP_PASS").is_err() {
        println!("Skipping real email test: SMTP_PASSWORD not set");
        return;
    }

    println!("Attempting to send direct test email...");

    // Check configuration
    let host = std::env::var("SMTP_HOST").unwrap_or_else(|_| "localhost".to_string());
    if host == "localhost" || host == "127.0.0.1" {
        println!("WARNING: SMTP_HOST is set to localhost. Real delivery might fail.");
    }

    // 2. Create Service
    let email_service =
        LettreEmailService::new().map(Arc::new).expect("Failed to create email service");

    // 3. Define Recipient (Self-send for testing)
    let to_email = std::env::var("SMTP_FROM").unwrap_or_else(|_| "test@example.com".to_string());
    let recipient =
        Recipient { email: to_email.clone(), name: "Integration Test User".to_string() };

    println!("Sending email to: {}", to_email);

    // 4. Send Email
    let email_type = EmailType::Welcome("Integration User".to_string());

    match email_service.send(recipient, email_type).await {
        Ok(_) => println!("✅ Email sent successfully! Check your inbox."),
        Err(e) => panic!("❌ Failed to send email: {}", e),
    }
}

/// Test full email delivery flow via API (Full Integration)
#[tokio::test]
#[ignore]
async fn test_api_triggered_email_delivery() {
    // 1. Check credentials
    dotenvy::dotenv().ok();
    if std::env::var("SMTP_PASSWORD").is_err() && std::env::var("SMTP_PASS").is_err() {
        println!("Skipping real email test: SMTP_PASSWORD not set");
        return;
    }

    // 2. Setup Server with Real Email
    let server = TestServer::new_with_real_email().await;

    // 3. Register user (Triggers Confirmation Email)
    let email = std::env::var("SMTP_FROM").unwrap_or_else(|_| "test@example.com".to_string());
    let name = "Real API Tester";
    let password = "TestPassword123!";

    println!("Attempting to register user: {}", email);
    println!("This should trigger a confirmation email to {}", email);

    // Use raw request to inspect response directly if needed, or helper
    let response = server
        .client
        .post(format!("{}/api/auth/register", server.base_url))
        .json(&serde_json::json!({
            "email": email,
            "name": name,
            "password": password
        }))
        .send()
        .await
        .expect("Failed to send register request");

    // 4. Verify API Response
    assert!(
        response.status().is_success(),
        "Registration failed: {:?}",
        response.text().await
    );

    println!("✅ Registration successful. Email API flow executed without error.");
}
