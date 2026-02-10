use axum::{routing::get, Json, Router};
use serde_json::json;

/// Health check endpoint
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service is healthy", body = Object)
    ),
    tag = "health"
)]
pub async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "service": "axum_backend",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

/// Create health check routes
pub fn health_routes() -> Router {
    Router::new().route("/health", get(health_check))
}
