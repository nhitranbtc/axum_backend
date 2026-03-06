#![allow(dead_code)]

use axum_backend::infrastructure::database::create_scylla_session;
use axum_backend::presentation::routes::create_router;
use axum_prometheus::{metrics_exporter_prometheus::PrometheusHandle, PrometheusMetricLayer};
use reqwest::Client;
use serde_json::{json, Value};
use std::net::SocketAddr;
use std::sync::OnceLock;
use tokio::net::TcpListener;

use crate::common::assertions::assert_success;
use crate::common::mock::MockScylla;

static PROMETHEUS_COMPONENTS: OnceLock<(PrometheusMetricLayer<'static>, PrometheusHandle)> =
    OnceLock::new();
use axum_backend::config::scylla::ScyllaConfig;
use axum_backend::infrastructure::database::DbPool;

// ── Constants ──────────────────────────────────────────────────────────────────

const DEFAULT_JWT_SECRET: &str = "test_secret_must_be_at_least_32_bytes_long";
const JWT_ACCESS_EXPIRY_SECS: i64 = 3600;
const JWT_REFRESH_EXPIRY_SECS: i64 = 86400;
const JWT_ISSUER: &str = "test-issuer";
const JWT_AUDIENCE: &str = "test-audience";
const CONFIRM_CODE_EXPIRY_SECS: i64 = 60;

// ── TestServer ─────────────────────────────────────────────────────────────────

/// A full in-process test server backed by an ephemeral ScyllaDB instance.
pub struct TestServer {
    pub addr: SocketAddr,
    pub client: Client,
    pub base_url: String,
    pub pool: DbPool,
    pub _mock_db: Option<MockScylla>,
}

impl TestServer {
    /// Create a server with a no-op email service (default for most tests).
    pub async fn new() -> Self {
        Self::build(false).await
    }

    /// Create a server that uses the real SMTP email service.
    pub async fn new_with_real_email() -> Self {
        Self::build(true).await
    }

    // ── Private builder ────────────────────────────────────────────────────────

    async fn build(use_real_email: bool) -> Self {
        dotenvy::dotenv().ok();

        // Ephemeral ScyllaDB — unique keyspace per test run for full isolation
        let mock_db = MockScylla::new().await;
        let scylla_config = ScyllaConfig {
            nodes: vec![mock_db.contact_node.clone()],
            keyspace: format!("test_keyspace_{}", uuid::Uuid::new_v4().simple()),
            username: Some(
                std::env::var("SCYLLA_USERNAME").unwrap_or_else(|_| "cassandra".to_string()),
            ),
            password: Some(
                std::env::var("SCYLLA_PASSWORD").unwrap_or_else(|_| "cassandra".to_string()),
            ),
            replication_factor: 1,
        };

        let jwt_secret =
            std::env::var("JWT_SECRET").unwrap_or_else(|_| DEFAULT_JWT_SECRET.to_string());

        let pool = create_scylla_session(&scylla_config)
            .await
            .expect("Failed to create test ScyllaDB session");
        let pool = std::sync::Arc::new(pool);

        let (prometheus_layer, metric_handle) =
            PROMETHEUS_COMPONENTS.get_or_init(|| PrometheusMetricLayer::pair()).clone();

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

        let cache_repository =
            std::sync::Arc::new(crate::common::repository_mocks::MockCacheRepository);

        let nats_client =
            axum_backend::infrastructure::messaging::NatsClient::new("127.0.0.1:4222")
                .await
                .expect("Failed to create test NATS client");

        let app = create_router(
            pool.clone(),
            jwt_secret,
            JWT_ACCESS_EXPIRY_SECS,
            JWT_REFRESH_EXPIRY_SECS,
            JWT_ISSUER.to_string(),
            JWT_AUDIENCE.to_string(),
            CONFIRM_CODE_EXPIRY_SECS,
            prometheus_layer,
            metric_handle,
            email_service,
            cache_repository,
            std::sync::Arc::new(nats_client),
        )
        .await;

        let listener = TcpListener::bind("127.0.0.1:0").await.expect("Failed to bind test server");
        let addr = listener.local_addr().expect("Failed to get local address");
        let base_url = format!("http://{}", addr);

        tokio::spawn(async move {
            axum::serve(listener, app).await.expect("Test server failed");
        });

        // Brief pause to let the server accept connections
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        Self {
            addr,
            client: Client::builder()
                .cookie_store(true)
                .build()
                .expect("Failed to build test client"),
            base_url,
            pool,
            _mock_db: Some(mock_db),
        }
    }

    // ── Low-level HTTP helpers ─────────────────────────────────────────────────

    /// POST a JSON body to `path` and deserialise the response.
    pub async fn post_json(&self, path: &str, body: Value) -> Value {
        self.client
            .post(format!("{}{}", self.base_url, path))
            .json(&body)
            .send()
            .await
            .unwrap_or_else(|e| panic!("POST {path} failed to send: {e}"))
            .json()
            .await
            .unwrap_or_else(|e| panic!("POST {path}: failed to parse JSON response: {e}"))
    }

    // ── Auth helpers ───────────────────────────────────────────────────────────

    /// Returns the 6-digit confirmation code for `email`.
    ///
    /// The test server uses a `NoOpEmailService`, so no real email is sent.
    /// The `MockScylla` node stores the code in the DB exactly as the
    /// production server does — but reading it back requires a live CQL query.
    ///
    pub async fn get_confirmation_code(&self, email: &str) -> String {
        let query = "SELECT confirmation_code FROM users WHERE email = ? ALLOW FILTERING";
        let values = (email,);

        let result = self
            .pool
            .session()
            .execute_unpaged(query, values)
            .await
            .unwrap_or_else(|e| panic!("Failed to query confirmation code for {email}: {e}"));

        let rows_result = result
            .into_rows_result()
            .unwrap_or_else(|e| panic!("Failed to decode confirmation code rows for {email}: {e}"));

        let mut rows = rows_result
            .rows::<(Option<String>,)>()
            .unwrap_or_else(|e| panic!("Failed to parse confirmation code row for {email}: {e}"));

        let code = rows
            .next()
            .unwrap_or_else(|| panic!("No user row found for email {email}"))
            .unwrap_or_else(|e| panic!("Failed to read confirmation code row for {email}: {e}"))
            .0
            .unwrap_or_else(|| panic!("Confirmation code is NULL for email {email}"));

        code
    }

    /// `POST /api/auth/verify` — verify an email with its confirmation code.
    pub async fn verify_email(&self, email: &str, code: &str) -> Value {
        let res = self
            .post_json("/api/auth/verify", json!({ "email": email, "code": code }))
            .await;
        assert_success(&res);
        res
    }

    /// `POST /api/auth/forgot-password` — request a new confirmation code.
    pub async fn forgot_password(&self, email: &str) -> Value {
        let res = self.post_json("/api/auth/forgot-password", json!({ "email": email })).await;
        assert_success(&res);
        res
    }

    /// `POST /api/auth/password` — set a new password using a confirmation code.
    pub async fn set_password(&self, email: &str, code: &str, password: &str) -> Value {
        let res = self
            .post_json(
                "/api/auth/password",
                json!({ "email": email, "code": code, "password": password }),
            )
            .await;
        assert_success(&res);
        res
    }

    /// Register a user, verify their email, and log them in.
    ///
    /// Uses the updated registration flow:
    /// `POST /api/auth/register` (with password)  
    /// → `POST /api/auth/verify`  
    /// → `POST /api/auth/login`
    ///
    /// Returns the parsed login response containing `access_token` and user info.
    pub async fn register_user(&self, email: &str, name: &str, password: &str) -> Value {
        // Register with password stored at creation time
        let reg = self
            .post_json(
                "/api/auth/register",
                json!({ "email": email, "name": name, "password": password }),
            )
            .await;
        assert_success(&reg);

        // Verify email with stub code
        let code = self.get_confirmation_code(email).await;
        self.verify_email(email, &code).await;

        // Login and return the full response for callers that inspect tokens
        let login = self
            .post_json("/api/auth/login", json!({ "email": email, "password": password }))
            .await;
        assert_success(&login);
        login
    }

    /// Login a user and return the `access_token` string.
    ///
    /// Checks the `access_token` cookie first, then falls back to the JSON body.
    pub async fn login_user(&self, email: &str, password: &str) -> String {
        let response = self
            .client
            .post(format!("{}/api/auth/login", self.base_url))
            .json(&json!({ "email": email, "password": password }))
            .send()
            .await
            .expect("Failed to send login request");

        let cookie_token = response
            .cookies()
            .find(|c| c.name() == "access_token")
            .map(|c| c.value().to_string());

        let body: Value = response.json().await.expect("Failed to parse login response");

        if let Some(token) = cookie_token.filter(|t| !t.is_empty()) {
            return token;
        }

        body.get("data")
            .and_then(|d| d.get("access_token"))
            .and_then(|t| t.as_str())
            .unwrap_or_else(|| panic!("No access_token in login response: {body:?}"))
            .to_string()
    }

    // ── Other helpers ──────────────────────────────────────────────────────────

    /// `GET /health`
    pub async fn health_check(&self) -> reqwest::Response {
        self.client
            .get(format!("{}/health", self.base_url))
            .send()
            .await
            .expect("Failed to send health check request")
    }

    /// `GET /api/users` with Bearer token authentication.
    pub async fn list_users(&self, token: &str, page: u32, page_size: u32) -> Value {
        self.client
            .get(format!("{}/api/users", self.base_url))
            .query(&[("page", page.to_string()), ("page_size", page_size.to_string())])
            .header("Authorization", format!("Bearer {token}"))
            .send()
            .await
            .expect("Failed to send list users request")
            .json()
            .await
            .expect("Failed to parse list users response")
    }
}
