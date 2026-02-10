use crate::infrastructure::monitoring::SystemMetrics;
use crate::infrastructure::SystemMonitor;
use axum::{Extension, Json};
use std::sync::Arc;

pub async fn system_health(
    Extension(monitor): Extension<Arc<SystemMonitor>>,
) -> Json<SystemMetrics> {
    Json(monitor.get_metrics())
}
