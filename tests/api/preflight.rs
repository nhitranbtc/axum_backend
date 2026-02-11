/// Pre-flight tests - Run these before starting the server
/// These tests validate configuration, database, and core functionality
use crate::common::*;

use axum_backend::config::AppConfig;
use axum_backend::infrastructure::database::connection::{create_pool, run_migrations};
use axum_backend::shared::utils::jwt::JwtManager;
use uuid::Uuid;

// =========================================================================
// Tests
// =========================================================================

#[tokio::test]

async fn check_database_connectivity() {
    dotenvy::dotenv().ok();

    // Standalone: Use Mock Postgres
    let mock_db = crate::common::mock::MockPostgres::new().await;
    let database_url = mock_db.connection_string.clone();

    // Use default config but override URL
    let config = axum_backend::config::DatabaseConfig::default();

    let pool = create_pool(&config, &database_url).await;
    assert!(pool.is_ok(), "Failed to create database pool: {:?}", pool.err());

    let pool = pool.unwrap();
    let conn = pool.get().await;
    assert!(conn.is_ok(), "Failed to get database connection: {:?}", conn.err());

    println!("✅ Database connection successful");
}

#[tokio::test]
async fn check_migrations_and_schema() {
    dotenvy::dotenv().ok();

    // Standalone: Use Mock Postgres
    let mock_db = crate::common::mock::MockPostgres::new().await;
    let database_url = mock_db.connection_string.clone();
    let config = axum_backend::config::DatabaseConfig::default();

    let pool = create_pool(&config, &database_url).await.expect("Failed to create pool");

    // Enable required extensions
    {
        use diesel::sql_query;
        use diesel_async::RunQueryDsl;
        let mut conn = pool.get().await.expect("Failed to get connection for setup");

        let _ = sql_query("CREATE EXTENSION IF NOT EXISTS \"uuid-ossp\";")
            .execute(&mut conn)
            .await;
        let _ = sql_query("CREATE EXTENSION IF NOT EXISTS \"pgcrypto\";")
            .execute(&mut conn)
            .await;
    }

    // Run migrations
    run_migrations(&database_url).await.expect("Failed to run migrations");
    println!("✅ Database migrations completed");

    // Check schema (users table)
    let mut conn = pool.get().await.expect("Failed to get connection");

    use axum_backend::infrastructure::database::schema::users;
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;

    let result: Result<i64, _> = users::table.count().get_result(&mut conn).await;
    assert!(result.is_ok(), "Users table does not exist or cannot be queried");

    println!("✅ Database schema validated");
}

#[test]
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
