use axum::{Extension, extract::State};
use plannify_driver_api_core::domain::driver::{
    entities::{CreateDriverRestPeriodsRequest, DriverRestPeriod},
    port::DriverService,
};

use crate::{
    ApiError, AppState,
    http::common::{
        api_error::ErrorBody, middleware::auth::entities::UserIdentity, response::Response,
        validator::ValidatedJson,
    },
};

#[utoipa::path(
    get,
    path = "/rest-periods",
    tag = "driver/rest-periods",
    description = "Retrieve driver rest periods",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Driver rest periods retrieved successfully", body = Vec<DriverRestPeriod>),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn get_all_rest_periods(
    State(state): State<AppState>,
    Extension(user_identity): Extension<UserIdentity>,
) -> Result<Response<Vec<DriverRestPeriod>>, ApiError> {
    let rest_periods = state
        .service
        .get_driver_rest_periods(user_identity.user_id)
        .await?;

    Ok(Response::ok(rest_periods))
}

#[utoipa::path(
    post,
    path = "/rest-periods",
    tag = "driver/rest-periods",
    description = "Set driver rest periods",
    request_body = Vec<DriverRestPeriod>,
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 201, description = "Driver rest periods set successfully"),
        (status = 400, description = "Invalid rest period", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn set_rest_periods(
    State(state): State<AppState>,
    Extension(user_identity): Extension<UserIdentity>,
    ValidatedJson(request): ValidatedJson<CreateDriverRestPeriodsRequest>,
) -> Result<Response<()>, ApiError> {
    state
        .service
        .set_driver_rest_periods(user_identity.user_id, request.rest_periods)
        .await?;

    Ok(Response::created(()))
}

#[utoipa::path(
    delete,
    path = "/rest-periods",
    tag = "driver/rest-periods",
    description = "Delete driver rest periods",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Driver rest periods deleted successfully"),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn delete_rest_periods(
    State(state): State<AppState>,
    Extension(user_identity): Extension<UserIdentity>,
) -> Result<Response<()>, ApiError> {
    state
        .service
        .delete_driver_rest_periods(user_identity.user_id)
        .await?;

    Ok(Response::ok(()))
}
