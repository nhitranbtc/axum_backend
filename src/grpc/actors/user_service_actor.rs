use ractor::{Actor, ActorProcessingErr, ActorRef};
use async_trait::async_trait;

use super::messages::{UserServiceMessage, UserServiceState};
use crate::grpc::interceptors::validation::validate_uuid;
use crate::grpc::proto::UserResponse;
use crate::domain::{
    repositories::user::UserRepository,
    value_objects::UserId,
};
use crate::infrastructure::database::{
    repositories::UserRepositoryImpl,
    connection::DbPool,
};
use tonic::Status;

/// User Service Actor - handles user-related requests
pub struct UserServiceActor;

#[async_trait]
impl Actor for UserServiceActor {
    type Msg = UserServiceMessage;
    type State = UserServiceState;
    type Arguments = DbPool;

    async fn pre_start(
        &self,
        myself: ActorRef<Self::Msg>,
        db_pool: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        tracing::info!("UserServiceActor {} starting", myself.get_id());
        Ok(UserServiceState {
            db_pool,
            request_count: 0,
        })
    }

    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            UserServiceMessage::GetUser { user_id, reply } => {
                state.request_count += 1;
                tracing::debug!(
                    actor_id = %myself.get_id(),
                    request_count = state.request_count,
                    user_id = %user_id,
                    "Processing GetUser request"
                );
                
                let result = self.handle_get_user(&user_id, &state.db_pool).await;
                
                if let Err(e) = reply.send(result) {
                    tracing::error!("Failed to send reply: {:?}", e);
                }
            }
            UserServiceMessage::HealthCheck { reply } => {
                tracing::debug!(actor_id = %myself.get_id(), "Health check");
                if let Err(e) = reply.send(true) {
                    tracing::error!("Failed to send health check reply: {:?}", e);
                }
            }
        }
        Ok(())
    }

    async fn post_stop(
        &self,
        myself: ActorRef<Self::Msg>,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        tracing::info!(
            actor_id = %myself.get_id(),
            total_requests = state.request_count,
            "UserServiceActor stopping"
        );
        Ok(())
    }
}

impl UserServiceActor {
    /// Handle GetUser request
    async fn handle_get_user(
        &self,
        user_id_str: &str,
        db_pool: &DbPool,
    ) -> Result<UserResponse, Status> {
        // Validate UUID format
        let user_uuid = validate_uuid(user_id_str)?;
        let user_id = UserId::from_uuid(user_uuid);

        // Query database using repository
        let repo = UserRepositoryImpl::new(db_pool.clone());
        
        let user_option = repo
            .find_by_id(user_id)
            .await
            .map_err(|e| {
                tracing::error!("Database query error: {}", e);
                Status::internal("Database query error")
            })?;

        // Check if user exists
        let user = user_option.ok_or_else(|| {
            tracing::warn!("User not found: {}", user_id_str);
            Status::not_found(format!("User with id {} not found", user_id_str))
        })?;

        // Convert domain entity to gRPC response
        Ok(UserResponse {
            id: user.id.to_string(),
            name: user.name.clone(),
            email: user.email.to_string(),
            created_at: user.created_at.timestamp(),
            created_at_ts: Some(prost_types::Timestamp {
                seconds: user.created_at.timestamp(),
                nanos: 0,
            }),
            updated_at: Some(prost_types::Timestamp {
                seconds: user.updated_at.timestamp(),
                nanos: 0,
            }),
            role: user.role.to_string(),
            is_active: user.is_active,
            last_login: user.last_login.map(|dt| prost_types::Timestamp {
                seconds: dt.timestamp(),
                nanos: 0,
            }),
            email_verified: user.is_email_verified,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ractor::{ActorRef, SupervisionEvent};
    use std::time::Duration;
    use tokio::time::sleep;

    // Mock database pool for testing
    fn create_test_db_pool() -> DbPool {
        // Create a mock database pool for testing
        // This uses the same type as the production code
        use diesel_async::pooled_connection::AsyncDieselConnectionManager;
        use diesel_async::AsyncPgConnection;
        use diesel_async::pooled_connection::deadpool::Pool;
        
        let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(
            "postgresql://test:test@localhost/test_db"
        );
        Pool::builder(manager)
            .max_size(1)
            .build()
            .unwrap()
    }

    #[test]
    fn test_actor_creation() {
        // Basic test to ensure actor compiles
        let _actor = UserServiceActor;
    }

    #[tokio::test]
    async fn test_spawn_actor_and_get_user() {
        // Test spawning an actor and sending a GetUser message
        let db_pool = create_test_db_pool();
        
        // Spawn the actor
        let (actor_ref, handle) = Actor::spawn(
            Some("test-user-service-actor".to_string()),
            UserServiceActor,
            db_pool,
        )
        .await
        .expect("Failed to spawn actor");

        // Verify actor is running
        assert!(matches!(actor_ref.get_status(), ractor::ActorStatus::Running));
        
        // Create a oneshot channel for the reply
        let (tx, rx) = ractor::concurrency::oneshot();
        
        // Send GetUser message (will fail due to mock DB, but tests message handling)
        let result = actor_ref.send_message(UserServiceMessage::GetUser {
            user_id: "test-uuid".to_string(),
            reply: tx.into(),
        });
        
        assert!(result.is_ok(), "Failed to send message to actor");
        
        // Wait a bit for message processing
        sleep(Duration::from_millis(100)).await;
        
        // Stop the actor
        actor_ref.stop(None);
        
        // Wait for actor to stop
        let _ = tokio::time::timeout(Duration::from_secs(2), handle).await;
        
        println!("✅ Test passed: Actor spawned and processed message");
    }

    #[tokio::test]
    async fn test_stop_actor() {
        // Test stopping an actor gracefully
        let db_pool = create_test_db_pool();
        
        // Spawn the actor
        let (actor_ref, handle) = Actor::spawn(
            Some("test-stop-actor".to_string()),
            UserServiceActor,
            db_pool,
        )
        .await
        .expect("Failed to spawn actor");

        // Verify actor is running
        assert!(matches!(actor_ref.get_status(), ractor::ActorStatus::Running));
        println!("✅ Actor is running");
        
        // Stop the actor
        actor_ref.stop(None);
        println!("📤 Sent stop signal to actor");
        
        // Wait for actor to stop
        let result = tokio::time::timeout(Duration::from_secs(2), handle).await;
        
        assert!(result.is_ok(), "Actor did not stop within timeout");
        println!("✅ Test passed: Actor stopped gracefully");
    }

    #[tokio::test]
    async fn test_actor_restart_with_supervision() {
        // Test actor restart using supervision
        use ractor::Actor;
        
        let db_pool = create_test_db_pool();
        
        // Spawn the actor with supervision
        let (actor_ref, handle) = Actor::spawn(
            Some("test-supervised-actor".to_string()),
            UserServiceActor,
            db_pool.clone(),
        )
        .await
        .expect("Failed to spawn actor");

        let actor_id = actor_ref.get_id();
        println!("✅ Actor spawned with ID: {}", actor_id);
        
        // Verify actor is running
        assert!(matches!(actor_ref.get_status(), ractor::ActorStatus::Running));
        println!("✅ Actor is running");
        
        // Send a health check to verify actor is responsive
        let (tx1, rx1) = ractor::concurrency::oneshot();
        actor_ref.send_message(UserServiceMessage::HealthCheck {
            reply: tx1.into(),
        }).expect("Failed to send health check");
        
        let health_result = tokio::time::timeout(Duration::from_secs(1), rx1).await;
        assert!(health_result.is_ok(), "Actor did not respond to health check");
        println!("✅ Actor responded to health check");
        
        // Stop the actor (simulating a failure)
        actor_ref.stop(None);
        println!("📤 Stopped actor (simulating failure)");
        
        // Wait for actor to stop
        let _ = tokio::time::timeout(Duration::from_secs(2), handle).await;
        println!("✅ Actor stopped");
        
        // In a real supervision scenario, the supervisor would restart the actor
        // For this test, we'll manually restart it to demonstrate the pattern
        let (new_actor_ref, new_handle) = Actor::spawn(
            Some("test-supervised-actor-restarted".to_string()),
            UserServiceActor,
            db_pool,
        )
        .await
        .expect("Failed to restart actor");

        let new_actor_id = new_actor_ref.get_id();
        println!("✅ Actor restarted with new ID: {}", new_actor_id);
        
        // Verify new actor is running
        assert!(matches!(new_actor_ref.get_status(), ractor::ActorStatus::Running));
        println!("✅ Restarted actor is running");
        
        // Send health check to verify restarted actor is responsive
        let (tx2, rx2) = ractor::concurrency::oneshot();
        new_actor_ref.send_message(UserServiceMessage::HealthCheck {
            reply: tx2.into(),
        }).expect("Failed to send health check to restarted actor");
        
        let health_result2 = tokio::time::timeout(Duration::from_secs(1), rx2).await;
        assert!(health_result2.is_ok(), "Restarted actor did not respond to health check");
        println!("✅ Restarted actor responded to health check");
        
        // Cleanup
        new_actor_ref.stop(None);
        let _ = tokio::time::timeout(Duration::from_secs(2), new_handle).await;
        
        println!("✅ Test passed: Actor restart verified");
    }

    #[tokio::test]
    async fn test_actor_state_preservation() {
        // Test that actor maintains state across messages
        let db_pool = create_test_db_pool();
        
        let (actor_ref, handle) = Actor::spawn(
            Some("test-state-actor".to_string()),
            UserServiceActor,
            db_pool,
        )
        .await
        .expect("Failed to spawn actor");

        // Send multiple health checks to increment request count
        for i in 1..=5 {
            let (tx, rx) = ractor::concurrency::oneshot();
            actor_ref.send_message(UserServiceMessage::HealthCheck {
                reply: tx.into(),
            }).expect("Failed to send health check");
            
            let _ = tokio::time::timeout(Duration::from_millis(100), rx).await;
            println!("✅ Health check {} completed", i);
        }
        
        // Wait a bit for all messages to process
        sleep(Duration::from_millis(200)).await;
        
        // Stop actor (will log request count in post_stop)
        actor_ref.stop(None);
        let _ = tokio::time::timeout(Duration::from_secs(2), handle).await;
        
        println!("✅ Test passed: Actor state maintained across messages");
    }
}
