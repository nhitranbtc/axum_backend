/// Service Registry
///
/// Centralized registry for managing actor pools and providing access to actors
use ractor::ActorRef;
use std::sync::Arc;

use crate::grpc::infrastructure::actors::handlers::user_actor::UserServiceActor;
use crate::grpc::infrastructure::actors::messages::user_messages::UserServiceMessage;
use crate::grpc::infrastructure::actors::pool::actor_pool::ActorPool;
use crate::infrastructure::database::connection::DbPool;

/// Service registry for managing actor pools
pub struct ServiceRegistry {
    /// User actor pool
    user_pool: Arc<ActorPool<UserServiceActor>>,

    /// Database pool (for creating new actors)
    db_pool: DbPool,
}

impl ServiceRegistry {
    /// Create a new service registry
    pub async fn new(
        db_pool: DbPool,
        user_pool_size: usize,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Create user actor pool
        let user_pool = ActorPool::new(
            user_pool_size,
            || UserServiceActor,
            db_pool.clone(),
            "user-service-worker",
        )
        .await?;

        tracing::info!("Service registry initialized with {} user actors", user_pool_size);

        Ok(Self { user_pool: Arc::new(user_pool), db_pool })
    }

    pub fn get_user_actor(&self) -> ActorRef<UserServiceMessage> {
        self.user_pool.next_worker()
    }

    /// Get user pool size
    pub fn user_pool_size(&self) -> usize {
        self.user_pool.size()
    }

    /// Shutdown all actor pools gracefully
    pub async fn shutdown(&self) {
        tracing::info!("Shutting down service registry");
        self.user_pool.shutdown().await;
        tracing::info!("Service registry shutdown complete");
    }
}

impl Clone for ServiceRegistry {
    fn clone(&self) -> Self {
        Self { user_pool: Arc::clone(&self.user_pool), db_pool: self.db_pool.clone() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Integration tests would require a test database
    #[test]
    fn test_service_registry_creation() {
        // Basic compilation test
        // Real tests would need database setup
    }
}
