/// gRPC Error Types
///
/// Provides a type-safe error hierarchy for gRPC operations with automatic
/// conversion to tonic::Status codes.
use thiserror::Error;
use tonic::Status;

/// Main gRPC error type
#[derive(Debug, Error)]
pub enum GrpcError {
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Already exists: {0}")]
    AlreadyExists(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Unauthenticated: {0}")]
    Unauthenticated(String),

    #[error("Resource exhausted: {0}")]
    ResourceExhausted(String),

    #[error("Failed precondition: {0}")]
    FailedPrecondition(String),

    #[error("Aborted: {0}")]
    Aborted(String),

    #[error("Out of range: {0}")]
    OutOfRange(String),

    #[error("Unimplemented: {0}")]
    Unimplemented(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Unavailable: {0}")]
    Unavailable(String),

    #[error("Data loss: {0}")]
    DataLoss(String),

    #[error("Deadline exceeded: {0}")]
    DeadlineExceeded(String),

    #[error("Cancelled: {0}")]
    Cancelled(String),
}

impl From<GrpcError> for Status {
    fn from(err: GrpcError) -> Self {
        match err {
            GrpcError::InvalidArgument(msg) => Status::invalid_argument(msg),
            GrpcError::NotFound(msg) => Status::not_found(msg),
            GrpcError::AlreadyExists(msg) => Status::already_exists(msg),
            GrpcError::PermissionDenied(msg) => Status::permission_denied(msg),
            GrpcError::Unauthenticated(msg) => Status::unauthenticated(msg),
            GrpcError::ResourceExhausted(msg) => Status::resource_exhausted(msg),
            GrpcError::FailedPrecondition(msg) => Status::failed_precondition(msg),
            GrpcError::Aborted(msg) => Status::aborted(msg),
            GrpcError::OutOfRange(msg) => Status::out_of_range(msg),
            GrpcError::Unimplemented(msg) => Status::unimplemented(msg),
            GrpcError::Internal(msg) => Status::internal(msg),
            GrpcError::Unavailable(msg) => Status::unavailable(msg),
            GrpcError::DataLoss(msg) => Status::data_loss(msg),
            GrpcError::DeadlineExceeded(msg) => Status::deadline_exceeded(msg),
            GrpcError::Cancelled(msg) => Status::cancelled(msg),
        }
    }
}

/// Helper methods for creating common errors
impl GrpcError {
    pub fn invalid_uuid(field: &str, value: &str) -> Self {
        Self::InvalidArgument(format!("Invalid UUID for {}: {}", field, value))
    }

    pub fn required_field(field: &str) -> Self {
        Self::InvalidArgument(format!("Required field missing: {}", field))
    }

    pub fn user_not_found(user_id: &str) -> Self {
        Self::NotFound(format!("User not found: {}", user_id))
    }

    pub fn token_not_found(token_id: &str) -> Self {
        Self::NotFound(format!("Token not found: {}", token_id))
    }

    pub fn email_already_exists(email: &str) -> Self {
        Self::AlreadyExists(format!("Email already registered: {}", email))
    }

    pub fn actor_unavailable(actor_type: &str) -> Self {
        Self::Unavailable(format!("{} actor unavailable", actor_type))
    }

    pub fn timeout(operation: &str) -> Self {
        Self::DeadlineExceeded(format!("Operation timed out: {}", operation))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_to_status_conversion() {
        let err = GrpcError::NotFound("test".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::NotFound);
    }

    #[test]
    fn test_helper_methods() {
        let err = GrpcError::invalid_uuid("user_id", "invalid");
        assert!(err.to_string().contains("Invalid UUID"));

        let err = GrpcError::user_not_found("123");
        assert!(err.to_string().contains("User not found"));
    }
}
