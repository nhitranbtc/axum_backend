use ractor::RpcReplyPort;
use tonic::Status;
use crate::grpc::proto::UserResponse;
use crate::infrastructure::database::connection::DbPool;

/// Messages that UserServiceActor can receive
#[derive(Debug)]
pub enum UserServiceMessage {
    /// Get user by ID
    GetUser {
        user_id: String,
        reply: RpcReplyPort<Result<UserResponse, Status>>,
    },
    
    /// Health check
    HealthCheck {
        reply: RpcReplyPort<bool>,
    },
}

/// Actor state
#[derive(Clone)]
pub struct UserServiceState {
    pub db_pool: DbPool,
    pub request_count: u64,
}
