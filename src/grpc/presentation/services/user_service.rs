use std::sync::Arc;
use tonic::{Request, Response, Status};

use super::super::super::proto::{user_service_server::UserService, *};
use crate::grpc::infrastructure::actors::{
    handlers::user_actor::UserServiceActor, messages::user_messages::UserServiceMessage,
    pool::actor_pool::ActorPool,
};
use crate::infrastructure::database::connection::DbPool;

/// gRPC User Service implementation with Actor Model
#[derive(Clone)]
pub struct UserServiceImpl {
    actor_pool: Arc<ActorPool<UserServiceActor>>,
}

impl UserServiceImpl {
    /// Create a new UserServiceImpl with an actor pool
    pub async fn new(
        db_pool: DbPool,
        pool_size: usize,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let actor_name_prefix = format!("user-service-worker-{}", uuid::Uuid::new_v4());
        let actor_pool =
            ActorPool::new(pool_size, || UserServiceActor, db_pool, &actor_name_prefix).await?;
        Ok(Self { actor_pool: Arc::new(actor_pool) })
    }

    /// Get the actor pool size
    pub fn pool_size(&self) -> usize {
        self.actor_pool.size()
    }

    /// Shutdown the actor pool gracefully
    pub async fn shutdown(&self) {
        self.actor_pool.shutdown().await;
    }
}

#[tonic::async_trait]
impl UserService for UserServiceImpl {
    async fn get_user(
        &self,
        request: Request<GetUserRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        let req = request.into_inner();
        let user_id = req.user_id.clone();

        tracing::info!("Received GetUser request for user_id: {}", user_id);

        // Get next worker from pool using round-robin
        let worker = self.actor_pool.next_worker();

        // Create RPC channel
        let (tx, rx) = ractor::concurrency::oneshot();

        // Send message to actor
        worker
            .send_message(UserServiceMessage::GetUser { user_id: req.user_id, reply: tx.into() })
            .map_err(|e| {
                tracing::error!("Failed to send message to actor: {:?}", e);
                Status::internal("Internal server error")
            })?;

        // Wait for reply with timeout
        let result = tokio::time::timeout(std::time::Duration::from_secs(5), rx)
            .await
            .map_err(|_| {
                tracing::error!("Actor call timed out for user_id: {}", user_id);
                Status::deadline_exceeded("Request timeout")
            })?
            .map_err(|_| {
                tracing::error!("Actor reply channel closed for user_id: {}", user_id);
                Status::internal("Internal server error")
            })??;

        tracing::info!("Returning user: {}", result.name);
        Ok(Response::new(result))
    }

    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        let req = request.into_inner();
        let email = req.email.clone();
        
        tracing::info!("Received CreateUser request for email: {}", email);

        let worker = self.actor_pool.next_worker();
        let (tx, rx) = ractor::concurrency::oneshot();

        worker.send_message(UserServiceMessage::CreateUser {
            email: req.email,
            name: req.name,
            password: req.password,
            role: req.role,
            reply: tx.into(),
        }).map_err(|e| {
            tracing::error!("Failed to send message to actor: {:?}", e);
            Status::internal("Internal server error")
        })?;

        let result = tokio::time::timeout(std::time::Duration::from_secs(5), rx)
            .await
            .map_err(|_| {
                Status::deadline_exceeded("Request timeout")
            })?
            .map_err(|_| {
                Status::internal("Internal server error")
            })??;

        tracing::info!("Created user: {}", result.id);
        Ok(Response::new(result))
    }

    async fn update_user(
        &self,
        _request: Request<UpdateUserRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        Err(Status::unimplemented("update_user not yet implemented"))
    }

    async fn delete_user(
        &self,
        _request: Request<DeleteUserRequest>,
    ) -> Result<Response<()>, Status> {
        Err(Status::unimplemented("delete_user not yet implemented"))
    }

    async fn list_users(
        &self,
        _request: Request<ListUsersRequest>,
    ) -> Result<Response<ListUsersResponse>, Status> {
        Err(Status::unimplemented("list_users not yet implemented"))
    }

    async fn verify_email(
        &self,
        _request: Request<VerifyEmailRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        Err(Status::unimplemented("verify_email not yet implemented"))
    }

    async fn resend_confirmation_code(
        &self,
        _request: Request<ResendConfirmationCodeRequest>,
    ) -> Result<Response<()>, Status> {
        Err(Status::unimplemented("resend_confirmation_code not yet implemented"))
    }

    async fn update_password(
        &self,
        _request: Request<UpdatePasswordRequest>,
    ) -> Result<Response<()>, Status> {
        Err(Status::unimplemented("update_password not yet implemented"))
    }

    async fn activate_user(
        &self,
        _request: Request<ActivateUserRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        Err(Status::unimplemented("activate_user not yet implemented"))
    }

    async fn deactivate_user(
        &self,
        _request: Request<DeactivateUserRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        Err(Status::unimplemented("deactivate_user not yet implemented"))
    }
}

#[cfg(test)]
mod tests {
    // Note: Integration tests would require a test database and actor runtime
    // Unit tests can mock the actor pool
}
