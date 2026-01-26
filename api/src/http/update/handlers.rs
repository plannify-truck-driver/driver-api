use axum::extract::State;
use plannify_driver_api_core::domain::update::{
    entities::{GetUpdatesByVersionParams, Update},
    port::UpdateService,
};

use crate::{
    ApiError, AppState,
    http::common::{
        api_error::ErrorBody,
        response::{PaginatedResponse, Response},
        validator::ValidatedQuery,
    },
};

#[utoipa::path(
    get,
    path = "/updates",
    tag = "updates",
    description = "Retrieve driver updates",
    params(GetUpdatesByVersionParams),
    responses(
        (status = 200, description = "Updates retrieved successfully", body = PaginatedResponse<Update>),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn get_all_updates_by_version(
    ValidatedQuery(query): ValidatedQuery<GetUpdatesByVersionParams>,
    State(state): State<AppState>,
) -> Result<Response<PaginatedResponse<Update>>, ApiError> {
    let (updates, total) = state
        .service
        .get_updates_by_version(query.version, query.page, query.limit)
        .await?;

    let response_updates: PaginatedResponse<Update> = PaginatedResponse {
        data: updates.iter().map(|u| u.to_update()).collect(),
        total,
        page: query.page,
    };

    Ok(Response::ok(response_updates))
}
