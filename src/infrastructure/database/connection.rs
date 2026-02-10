use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::AsyncPgConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

/// Database pool type
pub type DbPool = Pool<AsyncPgConnection>;

use crate::config::database::DatabaseConfig;

/// Create a database connection pool using diesel-async
pub async fn create_pool(config: &DatabaseConfig, database_url: &str) -> anyhow::Result<DbPool> {
    Ok(config.create_pool(database_url))
}

/// Run database migrations using synchronous diesel (as diesel_migrations requires it)
pub async fn run_migrations(database_url: &str) -> anyhow::Result<()> {
    use diesel::pg::PgConnection;
    use diesel::Connection;

    let mut conn = PgConnection::establish(database_url)
        .map_err(|e| anyhow::anyhow!("Failed to connect to database for migrations: {}", e))?;

    conn.run_pending_migrations(MIGRATIONS)
        .map_err(|e| anyhow::anyhow!("Failed to run migrations: {}", e))?;

    tracing::info!("Database migrations completed successfully");
    Ok(())
}
