use axum::body::Body;
use axum::http::response::Response;
use axum::http::{header, StatusCode};
use axum::Router;
use std::sync::Arc;
use tower_governor::{
    governor::GovernorConfigBuilder, key_extractor::SmartIpKeyExtractor, GovernorError,
    GovernorLayer,
};

/// Apply rate limiting to a router based on client IP.
///
/// Uses `SmartIpKeyExtractor` which checks `X-Forwarded-For` and `X-Real-IP`
/// headers before falling back to the peer socket address.
///
/// Returns HTTP 429 with JSON body and `Retry-After` header when the limit is exceeded.
pub fn apply_rate_limit(router: Router, per_second: u64, burst_size: u32) -> Router {
    // SAFETY: GovernorConfigBuilder only returns None when per_second is 0.
    // We validate at the config layer that per_second defaults to 2.
    #[allow(clippy::expect_used)]
    let config = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(per_second)
            .burst_size(burst_size)
            .key_extractor(SmartIpKeyExtractor)
            .error_handler(rate_limit_error_handler)
            .finish()
            .expect("GovernorConfig: per_second must be > 0"),
    );

    router.layer(GovernorLayer { config })
}

fn rate_limit_error_handler(error: GovernorError) -> Response<Body> {
    let (status, body_json) = match &error {
        GovernorError::TooManyRequests { wait_time, .. } => (
            StatusCode::TOO_MANY_REQUESTS,
            serde_json::json!({
                "success": false,
                "error": format!("Too many requests. Please try again in {}s.", wait_time)
            }),
        ),
        GovernorError::UnableToExtractKey => (
            StatusCode::INTERNAL_SERVER_ERROR,
            serde_json::json!({
                "success": false,
                "error": "Unable to identify client"
            }),
        ),
        GovernorError::Other { code, msg, .. } => (
            *code,
            serde_json::json!({
                "success": false,
                "error": msg.clone().unwrap_or_else(|| "Rate limit error".to_string())
            }),
        ),
    };

    let retry_after = match &error {
        GovernorError::TooManyRequests { wait_time, .. } => Some(wait_time.to_string()),
        _ => None,
    };

    let body_bytes = serde_json::to_vec(&body_json).unwrap_or_default();

    let mut builder = Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "application/json");

    if let Some(secs) = retry_after {
        builder = builder.header(header::RETRY_AFTER, secs);
    }

    builder.body(Body::from(body_bytes)).unwrap_or_else(|_| {
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from("Internal server error"))
            .unwrap_or_default()
    })
}
