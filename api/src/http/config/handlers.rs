use axum::extract::State;
use serde::Serialize;
use utoipa::ToSchema;

use crate::{
    ApiError, AppState,
    http::common::{api_error::ErrorBody, response::Response},
};

#[derive(Serialize, ToSchema)]
pub struct ConfigResponse {
    pub workday_garbage_retention_days: i64,
    pub support_email: String,
}

#[utoipa::path(
    get,
    path = "/config",
    tag = "config",
    description = "Get exposed application configuration",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Configuration retrieved successfully", body = ConfigResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn get_config(
    State(state): State<AppState>,
) -> Result<Response<ConfigResponse>, ApiError> {
    Ok(Response::ok(ConfigResponse {
        workday_garbage_retention_days: state.config.common.workday_garbage_retention_days,
        support_email: state.config.common.support_email.clone(),
    }))
}
