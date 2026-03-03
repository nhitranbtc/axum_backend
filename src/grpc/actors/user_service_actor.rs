use ractor::{Actor, ActorProcessingErr, ActorRef};
use async_trait::async_trait;

use super::messages::{UserServiceMessage, UserServiceState};
use crate::grpc::interceptors::validation::validate_uuid;
use crate::grpc::proto::UserResponse;
use crate::domain::{
    repositories::user::UserRepository,
    value_objects::UserId,
};
use crate::infrastructure::database::{DbPool, UserRepositoryImpl};
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
        let repo = UserRepositoryImpl::new(db_pool.clone())
            .await
            .map_err(|e| { tracing::error!("Repo init failed: {}", e); Status::internal("DB error") })?;
        
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

    #[test]
    fn test_actor_creation() {
        // Basic test to ensure actor compiles
        let _actor = UserServiceActor;
    }

    // Integration tests that spawn actors require a live ScyllaDB session.
    // Run them with: cargo test -- --ignored
}


