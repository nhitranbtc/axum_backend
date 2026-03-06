use anyhow::{Context, Result};
use scylla::client::caching_session::CachingSession;
use scylla::client::session::Session;
use scylla::client::session_builder::SessionBuilder;
use std::sync::Arc;
use tracing::info;

use crate::config::scylla::ScyllaConfig;

/// Number of entries kept in the CachingSession prepared-statement cache.
/// Covers all unique query strings used by every repository.
const PREPARED_CACHE_SIZE: usize = 64;

/// Wrapper around a ScyllaDB [`CachingSession`].
///
/// `CachingSession` automatically prepares and caches statements the first time
/// they are executed, eliminating the need for repository structs to carry
/// `PreparedStatement` fields (charybdis-style architecture).
#[derive(Clone)]
pub struct ScyllaSession {
    session: Arc<CachingSession>,
    keyspace: String,
}

impl ScyllaSession {
    /// Create a new ScyllaDB session.
    pub async fn new(config: &ScyllaConfig) -> Result<Self> {
        info!("Connecting to ScyllaDB nodes: {:?}", config.nodes);

        let mut builder = SessionBuilder::new().known_nodes(&config.nodes);

        if let (Some(username), Some(password)) = (&config.username, &config.password) {
            info!("Using ScyllaDB authentication");
            builder = builder.user(username, password);
        }

        let consistency = if config.replication_factor == 1 {
            scylla::statement::Consistency::LocalOne
        } else {
            scylla::statement::Consistency::LocalQuorum
        };

        let profile = scylla::client::execution_profile::ExecutionProfile::builder()
            .consistency(consistency)
            .build();

        builder = builder.default_execution_profile_handle(profile.into_handle());

        let raw_session: Session = builder
            .build()
            .await
            .context("Failed to connect to ScyllaDB")?;

        info!("Successfully connected to ScyllaDB");

        let scylla_session = Self {
            session: Arc::new(CachingSession::from(raw_session, PREPARED_CACHE_SIZE)),
            keyspace: config.keyspace.clone(),
        };

        scylla_session.initialize_schema(config).await?;

        Ok(scylla_session)
    }

    /// Returns a clone of the inner `Arc<CachingSession>`.
    ///
    /// Cloning an `Arc` is a cheap atomic reference-count increment.
    /// Repositories store this `Arc` directly — there is no second allocation.
    pub fn session(&self) -> Arc<CachingSession> {
        Arc::clone(&self.session)
    }

    /// Returns the keyspace name used by this session.
    pub fn keyspace(&self) -> &str {
        &self.keyspace
    }

    /// Health check — verifies the connection is alive.
    pub async fn health_check(&self) -> Result<()> {
        self.session
            .execute_unpaged("SELECT now() FROM system.local", &[])
            .await
            .context("ScyllaDB health check failed")?;
        Ok(())
    }

    // ── Schema initialisation ─────────────────────────────────────────────────

    async fn initialize_schema(&self, config: &ScyllaConfig) -> Result<()> {
        info!("Initialising ScyllaDB schema for keyspace '{}'", config.keyspace);

        self.create_keyspace(config).await?;
        self.use_keyspace(config).await?;
        self.create_tables().await?;
        self.create_indexes().await?;

        info!("ScyllaDB schema initialised successfully");
        Ok(())
    }

    async fn create_keyspace(&self, config: &ScyllaConfig) -> Result<()> {
        let cql = format!(
            "CREATE KEYSPACE IF NOT EXISTS {} \
             WITH replication = {{'class': 'NetworkTopologyStrategy', 'datacenter1': {}}} \
             AND TABLETS = {{'enabled': false}}",
            config.keyspace, config.replication_factor
        );
        self.session
            .execute_unpaged(cql.as_str(), &[])
            .await
            .context("Failed to create keyspace")?;
        info!("Keyspace '{}' created", config.keyspace);
        Ok(())
    }

    async fn use_keyspace(&self, config: &ScyllaConfig) -> Result<()> {
        // use_keyspace is on the raw Session; access it via CachingSession::get_session
        self.session
            .get_session()
            .use_keyspace(&config.keyspace, false)
            .await
            .context("Failed to USE keyspace")?;
        Ok(())
    }

    /// Creates all tables in parallel where dependency allows.
    async fn create_tables(&self) -> Result<()> {
        let s = &self.session;

        // user_events: drop old TIMEUUID schema, recreate with UUID
        s.execute_unpaged("DROP TABLE IF EXISTS user_events", &[])
            .await
            .context("Failed to drop user_events")?;

        let create_users = s.execute_unpaged(
            "CREATE TABLE IF NOT EXISTS users (\
                user_id     UUID PRIMARY KEY,\
                email       TEXT,\
                name        TEXT,\
                password_hash TEXT,\
                role        TEXT,\
                is_active   BOOLEAN,\
                email_verified BOOLEAN,\
                confirmation_code TEXT,\
                confirmation_code_expires_at TIMESTAMP,\
                last_login  TIMESTAMP,\
                created_at  TIMESTAMP,\
                updated_at  TIMESTAMP\
            )",
            &[],
        );

        let create_refresh_tokens = s.execute_unpaged(
            "CREATE TABLE IF NOT EXISTS refresh_tokens (\
                token_hash  TEXT PRIMARY KEY,\
                user_id     UUID,\
                expires_at  TIMESTAMP,\
                created_at  TIMESTAMP,\
                revoked_at  TIMESTAMP\
            )",
            &[],
        );

        let create_user_sessions = s.execute_unpaged(
            "CREATE TABLE IF NOT EXISTS user_sessions (\
                session_id UUID PRIMARY KEY,\
                user_id    UUID,\
                data       TEXT,\
                expires_at TIMESTAMP,\
                created_at TIMESTAMP\
            )",
            &[],
        );

        let create_user_events = s.execute_unpaged(
            "CREATE TABLE IF NOT EXISTS user_events (\
                user_id    UUID,\
                event_id   UUID,\
                event_type TEXT,\
                event_data TEXT,\
                created_at TIMESTAMP,\
                PRIMARY KEY (user_id, event_id)\
            )",
            &[],
        );

        tokio::try_join!(
            create_users,
            create_refresh_tokens,
            create_user_sessions,
            create_user_events,
        )
        .context("Failed to create tables")?;

        info!("All tables created");
        Ok(())
    }

    /// Creates secondary indexes after tables exist (sequential, order matters).
    async fn create_indexes(&self) -> Result<()> {
        let s = &self.session;

        s.execute_unpaged("CREATE INDEX IF NOT EXISTS ON users (email)", &[])
            .await
            .context("Failed to create users email index")?;

        s.execute_unpaged("CREATE INDEX IF NOT EXISTS ON refresh_tokens (user_id)", &[])
            .await
            .context("Failed to create refresh_tokens user_id index")?;

        tokio::try_join!(
            s.execute_unpaged("CREATE INDEX IF NOT EXISTS ON user_sessions (user_id)", &[]),
            s.execute_unpaged("CREATE INDEX IF NOT EXISTS ON user_sessions (expires_at)", &[]),
        )
        .context("Failed to create user_sessions indexes")?;

        info!("All indexes created");
        Ok(())
    }
}

/// Convenience constructor — used by the application bootstrap.
pub async fn create_scylla_session(config: &ScyllaConfig) -> Result<ScyllaSession> {
    ScyllaSession::new(config).await
}
