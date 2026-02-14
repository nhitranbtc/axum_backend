#![allow(dead_code)]

use axum_backend::infrastructure::database::connection::create_pool;
use axum_backend::infrastructure::database::schema::users;
use axum_backend::presentation::routes::create_router;
use axum_prometheus::{metrics_exporter_prometheus::PrometheusHandle, PrometheusMetricLayer};
use diesel::prelude::*;
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use reqwest::Client;
use serde_json::{json, Value};
use std::net::SocketAddr;
use std::sync::OnceLock;
use tokio::net::TcpListener;

static PROMETHEUS_COMPONENTS: OnceLock<(PrometheusMetricLayer, PrometheusHandle)> = OnceLock::new();

use crate::common::mock::MockPostgres;
use axum_backend::config::DatabaseConfig;

/// Test server instance
pub struct TestServer {
    pub addr: SocketAddr,
    pub client: Client,
    pub base_url: String,
    pub _mock_db: Option<MockPostgres>,
}

impl TestServer {
    /// Create a new test server instance
    pub async fn new() -> Self {
        Self::build(false).await
    }

    /// Create a new test server instance with real email service
    pub async fn new_with_real_email() -> Self {
        Self::build(true).await
    }

    async fn build(use_real_email: bool) -> Self {
        // 1. Initialize Infrastructure (Standalone)
        dotenvy::dotenv().ok();

        // Always use ephemeral database for tests to ensure isolation and independence
        let mock_db = MockPostgres::new().await;
        let db_url = mock_db.connection_string.clone();
        let mock_db = Some(mock_db);

        // Determine critical params, fallback if needed
        let jwt_secret = std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "test_secret_must_be_at_least_32_bytes_long".to_string());
        let jwt_access_expiry = 3600;
        let jwt_refresh_expiry = 86400;
        let jwt_issuer = "test-issuer".to_string();
        let jwt_audience = "test-audience".to_string();

        // 2. Create Database Pool
        // Allow overriding via env vars for load testing
        let max_connections = std::env::var("DB_MAX_CONNECTIONS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(10);

        let db_config = DatabaseConfig {
            max_connections,
            min_connections: 1,
            connect_timeout: std::time::Duration::from_secs(30),
            idle_timeout: std::time::Duration::from_secs(600),
            max_lifetime: std::time::Duration::from_secs(1800),
        };

        let pool = create_pool(&db_config, &db_url)
            .await
            .expect("Failed to create test database pool");

        // Enable required extensions regarding UUID generation
        {
            use diesel::sql_query;
            use diesel_async::RunQueryDsl;
            let mut conn = pool.get().await.expect("Failed to get connection for setup");

            // Try enabling pgcrypto and uuid-ossp
            let _ = sql_query("CREATE EXTENSION IF NOT EXISTS \"uuid-ossp\";")
                .execute(&mut conn)
                .await;
            let _ = sql_query("CREATE EXTENSION IF NOT EXISTS \"pgcrypto\";")
                .execute(&mut conn)
                .await;
        }

        // 3. Run Migrations (Idempotent)
        axum_backend::infrastructure::database::connection::run_migrations(&db_url)
            .await
            .expect("Failed to run migrations");

        // 4. Monitoring Setup (Singleton for tests to avoid panic)
        let (prometheus_layer, metric_handle) =
            PROMETHEUS_COMPONENTS.get_or_init(|| PrometheusMetricLayer::pair()).clone();

        // 5. Create Router
        let email_service: std::sync::Arc<
            dyn axum_backend::application::services::email::EmailService,
        > = if use_real_email {
            std::sync::Arc::new(
                axum_backend::infrastructure::email::lettre_service::LettreEmailService::new()
                    .expect("Failed to create real email service"),
            )
        } else {
            std::sync::Arc::new(
                axum_backend::infrastructure::email::noop_service::NoOpEmailService::new(),
            )
        };

        let cache_repository = std::sync::Arc::new(crate::common::repository_mocks::MockCacheRepository);

        let app = create_router(
            pool,
            jwt_secret,
            jwt_access_expiry,
            jwt_refresh_expiry,
            jwt_issuer,
            jwt_audience,
            60, // confirm_code_expiry
            prometheus_layer,
            metric_handle,
            email_service,
            cache_repository,
        );

        // 5. Bind to Random Port
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("Failed to bind test server");
        let addr = listener.local_addr().expect("Failed to get local address");
        let base_url = format!("http://{}", addr);

        // 6. Spawn Server Background Task
        tokio::spawn(async move {
            axum::serve(listener, app).await.expect("Test server failed");
        });

        // 7. Wait for readiness
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        Self {
            addr,
            client: Client::builder()
                .cookie_store(true)
                .build()
                .expect("Failed to build test client"),
            base_url,
            _mock_db: mock_db,
        }
    }

    /// Get confirmation code from DB
    pub async fn get_confirmation_code(&self, email_addr: &str) -> String {
        let db_url = &self._mock_db.as_ref().expect("Mock DB not initialized").connection_string;
        let mut conn = AsyncPgConnection::establish(db_url).await.expect("Failed to connect to DB");

        let code: Option<String> = users::table
            .filter(users::email.eq(email_addr))
            .select(users::confirmation_code)
            .first(&mut conn)
            .await
            .expect("Failed to query user");

        code.expect("Confirmation code not found")
    }

    /// Register a test user (Full Flow: Register -> Verify -> SetPassword -> Login)
    pub async fn register_user(&self, email: &str, name: &str, password: &str) -> Value {
        // 1. Register
        let register_res = self
            .client
            .post(format!("{}/api/auth/register", self.base_url))
            .json(&json!({
                "email": email,
                "name": name
                // password removed from request
            }))
            .send()
            .await
            .expect("Failed to send register request");

        if !register_res.status().is_success() {
            let status = register_res.status();
            let body = register_res.text().await.unwrap();
            panic!("Register failed: {} - {}", status, body);
        }

        // 2. Get Code
        let code = self.get_confirmation_code(email).await;

        // 3. Verify Email
        let verify_res = self
            .client
            .post(format!("{}/api/auth/verify", self.base_url))
            .json(&json!({
                "email": email,
                "code": code
            }))
            .send()
            .await
            .expect("Failed to send verify request");

        if !verify_res.status().is_success() {
            panic!("Verify failed: {:?}", verify_res.text().await);
        }

        // 4. Set Password
        let password_res = self
            .client
            .post(format!("{}/api/auth/password", self.base_url))
            .json(&json!({
                "email": email,
                "code": code,
                "password": password
            }))
            .send()
            .await
            .expect("Failed to send set password request");

        if !password_res.status().is_success() {
            panic!("Set password failed: {:?}", password_res.text().await);
        }

        // 5. Login to get tokens (backward compatibility)
        let login_res = self
            .client
            .post(format!("{}/api/auth/login", self.base_url))
            .json(&json!({
                "email": email,
                "password": password
            }))
            .send()
            .await
            .expect("Failed to send login request");

        login_res.json().await.expect("Failed to parse login response")
    }

    /// Login a user and return the access token
    pub async fn login_user(&self, email: &str, password: &str) -> String {
        let response = self
            .client
            .post(format!("{}/api/auth/login", self.base_url))
            .json(&json!({
                "email": email,
                "password": password
            }))
            .send()
            .await
            .expect("Failed to send login request");

        // Try to get token from cookie first
        let cookie_token = response
            .cookies()
            .find(|c| c.name() == "access_token")
            .map(|c| c.value().to_string());

        let response_json: Value = response.json().await.expect("Failed to parse login response");

        // If cookie found, use it. Otherwise rely on JSON body (legacy or failure case)
        if let Some(token) = cookie_token {
            if !token.is_empty() {
                return token;
            }
        }

        let token_option = response_json
            .get("data")
            .and_then(|data| data.get("access_token"))
            .and_then(|token| token.as_str());

        if token_option.is_none() {
            println!("❌ Login failed! Response: {:?}", response_json);
            panic!("No access token in response");
        }

        token_option.unwrap().to_string()
    }

    /// Get health check
    pub async fn health_check(&self) -> reqwest::Response {
        self.client
            .get(format!("{}/health", self.base_url))
            .send()
            .await
            .expect("Failed to send health check request")
    }

    /// List users with authentication
    pub async fn list_users(&self, token: &str, page: u32, page_size: u32) -> Value {
        self.client
            .get(format!("{}/api/users?page={}&page_size={}", self.base_url, page, page_size))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .expect("Failed to send list users request")
            .json()
            .await
            .expect("Failed to parse list users response")
    }
}
