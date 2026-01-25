use axum::extract::State;
use chrono::Utc;
use plannify_driver_api_core::domain::health::port::HealthService;
use serde::Serialize;
use utoipa::ToSchema;

use crate::http::common::{api_error::ApiError, app_state::AppState, response::Response};

/// Response structure for the health check
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub database_status: String,
    pub cache_status: String,
    pub timestamp: String,
}

pub async fn health_check(
    State(state): State<AppState>,
) -> Result<Response<HealthResponse>, ApiError> {
    let health_check = state.service.check_health().await?;

    let is_healthy = health_check.to_result().is_ok();
    let status = if is_healthy { "healthy" } else { "unhealthy" };
    let database_status = if health_check.database {
        "connected"
    } else {
        "disconnected"
    };
    let cache_status = if health_check.cache {
        "connected"
    } else {
        "disconnected"
    };

    let response = HealthResponse {
        status: status.to_string(),
        database_status: database_status.to_string(),
        cache_status: cache_status.to_string(),
        timestamp: Utc::now().to_rfc3339(),
    };

    Ok(Response::ok(response))
}
