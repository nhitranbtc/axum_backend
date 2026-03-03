/// Pre-flight tests - Run these before starting the server
/// These tests validate configuration, database, and core functionality
use crate::common::*;
use axum_backend::infrastructure::database::create_scylla_session;
use axum_backend::config::scylla::ScyllaConfig;
use axum_backend::shared::utils::jwt::JwtManager;
use uuid::Uuid;

// =========================================================================
// Tests
// =========================================================================

#[tokio::test]
#[ignore]
async fn check_database_connectivity() {
    dotenvy::dotenv().ok();

    // Standalone: Use Mock ScyllaDB
    let mock_db = crate::common::mock::MockScylla::new().await;
    
    let scylla_config = ScyllaConfig {
        nodes: vec![mock_db.contact_node.clone()],
        keyspace: format!("test_keyspace_{}", uuid::Uuid::new_v4().simple()),
        username: None,
        password: None,
        replication_factor: 1,
    };

    let session = create_scylla_session(&scylla_config).await;
    assert!(session.is_ok(), "Failed to create ScyllaDB session: {:?}", session.err());

    let session = session.unwrap();
    
    let health = session.health_check().await;
    assert!(health.is_ok(), "Failed to perform health check on ScyllaDB: {:?}", health.err());

    println!("✅ Database connection successful");
}

#[test]
#[ignore]
fn check_jwt_logic() {
    let jwt_manager = JwtManager::new(
        "test-secret-that-is-at-least-thirty-two-bytes-long".to_string(),
        3600,
        86400,
        "test-issuer".to_string(),
        "test-audience".to_string(),
    );

    let user_id = Uuid::new_v4();

    // Create tokens
    let access_token = jwt_manager.create_access_token(user_id);
    assert!(access_token.is_ok(), "Failed to create access token");

    let refresh_token = jwt_manager.create_refresh_token(user_id);
    assert!(refresh_token.is_ok(), "Failed to create refresh token");

    // Verify token
    let claims = jwt_manager.verify_token(&access_token.unwrap());
    assert!(claims.is_ok(), "Failed to verify token");

    let claims = claims.unwrap();
    assert_eq!(claims.sub, user_id.to_string());
    assert_eq!(claims.iss, "test-issuer");

    println!("✅ JWT functionality working");
}

#[test]
#[ignore]
fn check_crypto_logic() {
    use argon2::{
        password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
        Argon2,
    };
    use rand::rngs::OsRng;

    let argon2 = Argon2::default();
    let password = "TestPassword@123";
    let salt = SaltString::generate(&mut OsRng);

    // Hash
    let hash_result = argon2.hash_password(password.as_bytes(), &salt);
    assert!(hash_result.is_ok(), "Failed to hash password");
    let password_hash = hash_result.unwrap().to_string();

    // Verify
    let parsed_hash = PasswordHash::new(&password_hash);
    assert!(parsed_hash.is_ok(), "Failed to parse hash");

    let verify_result = argon2.verify_password(password.as_bytes(), &parsed_hash.unwrap());
    assert!(verify_result.is_ok(), "Failed to verify password");

    println!("✅ Password hashing working");
}
