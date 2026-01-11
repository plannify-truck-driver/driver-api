use axum::{
    Extension,
    extract::{Query, State},
};
use plannify_driver_api_core::domain::workday::{
    entities::{CreateWorkdayRequest, GetWorkdaysByMonthParams, Workday},
    port::WorkdayService,
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
    path = "/workdays/month",
    tag = "workdays",
    params(GetWorkdaysByMonthParams),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Monthly workdays retrieved successfully", body = Vec<Workday>),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn get_all_month(
    Query(query): Query<GetWorkdaysByMonthParams>,
    State(state): State<AppState>,
    Extension(user_identity): Extension<UserIdentity>,
) -> Result<Response<Vec<Workday>>, ApiError> {
    let workdays = state
        .service
        .get_workdays_by_month(user_identity.user_id, query.month, query.year)
        .await?;
    let response_workdays: Vec<Workday> = workdays.iter().map(|w| w.to_workday()).collect();

    Ok(Response::ok(response_workdays))
}

#[utoipa::path(
    post,
    path = "/workdays",
    tag = "workdays",
    request_body = CreateWorkdayRequest,
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 201, description = "Workday created successfully", body = Workday),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn create_workday(
    State(state): State<AppState>,
    Extension(user_identity): Extension<UserIdentity>,
    ValidatedJson(request): ValidatedJson<CreateWorkdayRequest>,
) -> Result<Response<Workday>, ApiError> {
    let workday = state
        .service
        .create_workday(user_identity.user_id, request)
        .await?;

    Ok(Response::created(workday.to_workday()))
}
