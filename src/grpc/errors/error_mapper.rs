use super::grpc_error::GrpcError;
/// Error Mapper
///
/// Maps domain errors to gRPC errors
use crate::domain::errors::DomainError;
use crate::domain::repositories::user_repository::RepositoryError;

/// Map domain errors to gRPC errors
impl From<DomainError> for GrpcError {
    fn from(err: DomainError) -> Self {
        match err {
            DomainError::InvalidEmail(msg) => {
                GrpcError::InvalidArgument(format!("Invalid email: {}", msg))
            },
            DomainError::InvalidName => {
                GrpcError::InvalidArgument("Invalid name: name cannot be empty".to_string())
            },
            DomainError::InvalidPassword(msg) => {
                GrpcError::InvalidArgument(format!("Invalid password: {}", msg))
            },
            DomainError::ValidationError(msg) => GrpcError::InvalidArgument(msg),
            DomainError::InvalidUserData(msg) => {
                GrpcError::InvalidArgument(format!("Invalid user data: {}", msg))
            },
        }
    }
}

/// Map repository errors to gRPC errors
impl From<RepositoryError> for GrpcError {
    fn from(err: RepositoryError) -> Self {
        match err {
            RepositoryError::NotFound => GrpcError::NotFound("Resource not found".to_string()),
            RepositoryError::AlreadyExists(msg) => {
                GrpcError::AlreadyExists(format!("Resource already exists: {}", msg))
            },
            RepositoryError::DatabaseError(msg) => {
                GrpcError::Internal(format!("Database error: {}", msg))
            },
            RepositoryError::ConnectionError(msg) => {
                GrpcError::Unavailable(format!("Database unavailable: {}", msg))
            },
            RepositoryError::Internal(msg) => GrpcError::Internal(msg),
        }
    }
}

/// Map actor errors to gRPC errors
impl From<ractor::MessagingErr<()>> for GrpcError {
    fn from(err: ractor::MessagingErr<()>) -> Self {
        GrpcError::Unavailable(format!("Actor messaging error: {:?}", err))
    }
}

/// Map timeout errors to gRPC errors
impl From<tokio::time::error::Elapsed> for GrpcError {
    fn from(_: tokio::time::error::Elapsed) -> Self {
        GrpcError::DeadlineExceeded("Request timeout".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_error_mapping() {
        let domain_err = DomainError::InvalidEmail("test".to_string());
        let grpc_err: GrpcError = domain_err.into();
        assert!(matches!(grpc_err, GrpcError::InvalidArgument(_)));
    }

    #[test]
    fn test_repository_error_mapping() {
        let repo_err = RepositoryError::NotFound;
        let grpc_err: GrpcError = repo_err.into();
        assert!(matches!(grpc_err, GrpcError::NotFound(_)));
    }
}
