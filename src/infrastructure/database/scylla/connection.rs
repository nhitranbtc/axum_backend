use anyhow::{Context, Result};
use scylla::client::session::Session;
use scylla::client::session_builder::SessionBuilder;
use std::sync::Arc;
use tracing::info;

use crate::config::scylla::ScyllaConfig;

/// Wrapper around ScyllaDB session with connection pooling
#[derive(Clone)]
pub struct ScyllaSession {
    session: Arc<Session>,
    keyspace: String,
}

impl ScyllaSession {
    /// Create a new ScyllaDB session
    pub async fn new(config: &ScyllaConfig) -> Result<Self> {
        info!("Connecting to ScyllaDB nodes: {:?}", config.nodes);

        let mut builder = SessionBuilder::new().known_nodes(&config.nodes);

        // Add authentication if provided
        if let (Some(username), Some(password)) = (&config.username, &config.password) {
            info!("Using ScyllaDB authentication");
            builder = builder.user(username, password);
        }

        let session = builder
            .build()
            .await
            .context("Failed to connect to ScyllaDB")?;

        info!("Successfully connected to ScyllaDB");

        let scylla_session = Self {
            session: Arc::new(session),
            keyspace: config.keyspace.clone(),
        };

        // Initialize keyspace and schema
        scylla_session.initialize_schema(config).await?;

        Ok(scylla_session)
    }

    /// Get reference to the underlying session
    pub fn session(&self) -> &Session {
        &self.session
    }

    /// Get the keyspace name
    pub fn keyspace(&self) -> &str {
        &self.keyspace
    }

    /// Initialize keyspace and schema
    async fn initialize_schema(&self, config: &ScyllaConfig) -> Result<()> {
        info!("Initializing ScyllaDB schema");

        // Create keyspace if it doesn't exist
        let create_keyspace = format!(
            "CREATE KEYSPACE IF NOT EXISTS {} WITH replication = {{'class': 'SimpleStrategy', 'replication_factor': {}}}",
            config.keyspace, config.replication_factor
        );

        self.session
            .query_unpaged(create_keyspace.as_str(), &[])
            .await
            .context("Failed to create keyspace")?;

        info!("Keyspace '{}' ready", config.keyspace);

        // Use the keyspace
        self.session
            .use_keyspace(&config.keyspace, false)
            .await
            .context("Failed to use keyspace")?;

        // Create tables
        self.create_tables().await?;

        info!("ScyllaDB schema initialized successfully");
        Ok(())
    }

    /// Create all required tables (runs DDL in parallel).
    async fn create_tables(&self) -> Result<()> {
        let s = self.session.as_ref();

        // ── Migrate user_events: drop old TIMEUUID schema, recreate with UUID ──
        s.query_unpaged("DROP TABLE IF EXISTS user_events", &[])
            .await
            .context("Failed to drop user_events table")?;

        s.query_unpaged(
            "CREATE TABLE IF NOT EXISTS user_events (
                user_id    UUID,
                event_id   UUID,
                event_type TEXT,
                event_data TEXT,
                created_at TIMESTAMP,
                PRIMARY KEY (user_id, event_id)
            )",
            &[],
        )
        .await
        .context("Failed to create user_events table")?;

        // ── Define all remaining DDL statements ────────────────────────────────
        let create_users = s.query_unpaged(
            "CREATE TABLE IF NOT EXISTS users (
                user_id     UUID PRIMARY KEY,
                email       TEXT,
                name        TEXT,
                password_hash TEXT,
                role        TEXT,
                is_active   BOOLEAN,
                email_verified BOOLEAN,
                confirmation_code TEXT,
                confirmation_code_expires_at TIMESTAMP,
                last_login  TIMESTAMP,
                created_at  TIMESTAMP,
                updated_at  TIMESTAMP
            )",
            &[],
        );

        let create_refresh_tokens = s.query_unpaged(
            "CREATE TABLE IF NOT EXISTS refresh_tokens (
                token_hash  TEXT PRIMARY KEY,
                user_id     UUID,
                expires_at  TIMESTAMP,
                created_at  TIMESTAMP,
                revoked_at  TIMESTAMP
            )",
            &[],
        );

        let create_user_sessions = s.query_unpaged(
            "CREATE TABLE IF NOT EXISTS user_sessions (
                session_id UUID PRIMARY KEY,
                user_id    UUID,
                data       TEXT,
                expires_at TIMESTAMP,
                created_at TIMESTAMP
            )",
            &[],
        );

        // ── Create remaining tables in parallel ────────────────────────────────
        tokio::try_join!(create_users, create_refresh_tokens, create_user_sessions)
            .context("Failed to create tables")?;
        info!("All tables created");

        // ── Indexes must follow after the tables exist ─────────────────────────
        // (Run these sequentially since they depend on the tables above)
        s.query_unpaged("CREATE INDEX IF NOT EXISTS ON users (email)", &[])
            .await
            .context("Failed to create users email index")?;

        s.query_unpaged("CREATE INDEX IF NOT EXISTS ON refresh_tokens (user_id)", &[])
            .await
            .context("Failed to create refresh_tokens user_id index")?;

        tokio::try_join!(
            s.query_unpaged("CREATE INDEX IF NOT EXISTS ON user_sessions (user_id)", &[]),
            s.query_unpaged("CREATE INDEX IF NOT EXISTS ON user_sessions (expires_at)", &[]),
        )
        .context("Failed to create user_sessions indexes")?;

        info!("All indexes created");
        Ok(())
    }

    /// Health check - verify connection is alive
    pub async fn health_check(&self) -> Result<()> {
        self.session
            .query_unpaged("SELECT now() FROM system.local", &[])
            .await
            .context("ScyllaDB health check failed")?;
        Ok(())
    }
}

/// Create a new ScyllaDB session from configuration
pub async fn create_scylla_session(config: &ScyllaConfig) -> Result<ScyllaSession> {
    ScyllaSession::new(config).await
}
