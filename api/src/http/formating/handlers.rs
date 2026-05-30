use plannify_driver_api_core::domain::{
    driver::port::{to_email_case, to_title_case},
    formating::entities::GetValueFormatingParams,
};

use crate::{
    ApiError,
    http::common::{api_error::ErrorBody, response::Response, validator::ValidatedQuery},
};

#[tracing::instrument(name = "format_first_name", skip_all)]
#[utoipa::path(
    get,
    path = "/formating/first-name",
    tag = "formating",
    description = "Format a first name to title case",
    security(),
    params(GetValueFormatingParams),
    responses(
        (status = 200, description = "First name formatted successfully", body = String),
        (status = 400, description = "Invalid query parameters", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
pub async fn get_first_name_formating(
    ValidatedQuery(query): ValidatedQuery<GetValueFormatingParams>,
) -> Result<Response<String>, ApiError> {
    let value = to_title_case(query.value);

    Ok(Response::ok(value))
}

#[tracing::instrument(name = "format_last_name", skip_all)]
#[utoipa::path(
    get,
    path = "/formating/last-name",
    tag = "formating",
    description = "Format a last name to title case",
    security(),
    params(GetValueFormatingParams),
    responses(
        (status = 200, description = "Last name formatted successfully", body = String),
        (status = 400, description = "Invalid query parameters", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
pub async fn get_last_name_formating(
    ValidatedQuery(query): ValidatedQuery<GetValueFormatingParams>,
) -> Result<Response<String>, ApiError> {
    let value = to_title_case(query.value);

    Ok(Response::ok(value))
}

#[tracing::instrument(name = "format_email", skip_all)]
#[utoipa::path(
    get,
    path = "/formating/email",
    tag = "formating",
    description = "Format an email to lower case",
    security(),
    params(GetValueFormatingParams),
    responses(
        (status = 200, description = "Email formatted successfully", body = String),
        (status = 400, description = "Invalid query parameters", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody),
    )
)]
pub async fn get_email_formating(
    ValidatedQuery(query): ValidatedQuery<GetValueFormatingParams>,
) -> Result<Response<String>, ApiError> {
    let value = to_email_case(query.value);

    Ok(Response::ok(value))
}
