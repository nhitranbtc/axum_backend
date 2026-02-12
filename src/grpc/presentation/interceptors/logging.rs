use tonic::{Request, Status};

/// Logging interceptor that logs incoming gRPC requests
///
/// This interceptor logs basic request information and assigns a request ID.
/// It can be used with `with_interceptor` when adding services to the server.
pub fn logging_interceptor<T>(mut req: Request<T>) -> Result<Request<T>, Status> {
    let remote_addr = req.remote_addr();

    tracing::info!(
        remote_addr = ?remote_addr,
        "Incoming gRPC request"
    );

    // Add request ID for tracing
    let request_id = uuid::Uuid::new_v4().to_string();
    req.metadata_mut().insert("x-request-id", request_id.parse().unwrap());

    tracing::debug!(request_id = %request_id, "Request ID assigned");

    Ok(req)
}

/// Response logging helper
///
/// Logs when a response is being sent back to the client
pub fn log_response(status: &str) {
    tracing::info!(
        status = %status,
        "Sending gRPC response"
    );
}
