use async_trait::async_trait;
use ractor::{Actor, ActorProcessingErr, ActorRef};

use super::super::messages::user_messages::{UserServiceMessage, UserServiceState};
use crate::domain::{
    entities::User,
    repositories::user::UserRepository,
    value_objects::{Email, UserId},
};
use crate::grpc::presentation::interceptors::validation::validate_uuid;
use crate::grpc::proto::UserResponse;
use crate::infrastructure::database::{connection::DbPool, repositories::UserRepositoryImpl};
use crate::shared::utils::password::PasswordManager;
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
        Ok(UserServiceState { db_pool, request_count: 0 })
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
            },
            UserServiceMessage::CreateUser { email, name, password, role, reply } => {
                state.request_count += 1;
                tracing::debug!(
                    actor_id = %myself.get_id(),
                    request_count = state.request_count,
                    email = %email,
                    "Processing CreateUser request"
                );

                let result = self.handle_create_user(email, name, password, role, &state.db_pool).await;

                if let Err(e) = reply.send(result) {
                    tracing::error!("Failed to send reply: {:?}", e);
                }
            },
            UserServiceMessage::HealthCheck { reply } => {
                tracing::debug!(actor_id = %myself.get_id(), "Health check");
                if let Err(e) = reply.send(true) {
                    tracing::error!("Failed to send health check reply: {:?}", e);
                }
            },
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

        let user_option = repo.find_by_id(user_id).await.map_err(|e| {
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
            last_login: user
                .last_login
                .map(|dt| prost_types::Timestamp { seconds: dt.timestamp(), nanos: 0 }),
            email_verified: user.is_email_verified,
        })
    }

    /// Handle CreateUser request
    async fn handle_create_user(
        &self,
        email_str: String,
        name: String,
        password_str: String,
        _role_str: Option<String>,
        db_pool: &DbPool,
    ) -> Result<UserResponse, Status> {
        // 1. Parse Email
        let email = Email::parse(&email_str).map_err(|e| {
            Status::invalid_argument(format!("Invalid email: {}", e))
        })?;

        // 2. Hash Password
        let password_hash = PasswordManager::hash(&password_str).map_err(|e| {
            tracing::error!("Password hashing failed: {}", e);
            Status::internal("Internal server error")
        })?;

        // 3. Create User Entity
        let mut user = User::new(email, name).map_err(|e| {
            Status::invalid_argument(format!("Invalid user data: {}", e))
        })?;
        user.set_password(password_hash);

        // 4. Save to Repository
        let repo = UserRepositoryImpl::new(db_pool.clone());
        
        let saved_user = repo.save(&user).await.map_err(|e| {
            match e {
                crate::domain::repositories::user::RepositoryError::AlreadyExists(_) => 
                    Status::already_exists(format!("Email {} already exists", email_str)),
                crate::domain::repositories::user::RepositoryError::DatabaseError(msg) => {
                    tracing::error!("Database error: {}", msg);
                    Status::internal("Database error")
                },
                _ => {
                    tracing::error!("Repository error: {}", e);
                    Status::internal("Internal server error")
                }
            }
        })?;

        // 5. Return Response
        Ok(UserResponse {
            id: saved_user.id.to_string(),
            name: saved_user.name.clone(),
            email: saved_user.email.to_string(),
            created_at: saved_user.created_at.timestamp(),
            created_at_ts: Some(prost_types::Timestamp {
                seconds: saved_user.created_at.timestamp(),
                nanos: 0,
            }),
            updated_at: Some(prost_types::Timestamp {
                seconds: saved_user.updated_at.timestamp(),
                nanos: 0,
            }),
            role: saved_user.role.to_string(),
            is_active: saved_user.is_active,
            last_login: saved_user.last_login.map(|dt| prost_types::Timestamp { seconds: dt.timestamp(), nanos: 0 }),
            email_verified: saved_user.is_email_verified,
        })
    }
}

// Tests mostly removed/simplified to avoid large file issues, as real tests are in tests/
