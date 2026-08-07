use axum::extract::State;
use plannify_driver_api_core::domain::driver_information::{
    entities::DriverInformation, port::DriverInformationService,
};

use crate::{
    ApiError, AppState,
    http::common::{api_error::ErrorBody, response::Response},
};

#[tracing::instrument(
    name = "get_driver_informations",
    skip_all,
    fields(count = tracing::field::Empty)
)]
#[utoipa::path(
    get,
    path = "/informations",
    tag = "informations",
    description = "Retrieve currently visible driver informations (announcements/warnings)",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Driver informations retrieved successfully", body = Vec<DriverInformation>),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn get_driver_informations(
    State(state): State<AppState>,
) -> Result<Response<Vec<DriverInformation>>, ApiError> {
    let informations = state.service.get_driver_informations().await?;

    tracing::Span::current().record("count", informations.len());

    Ok(Response::ok(informations))
}
