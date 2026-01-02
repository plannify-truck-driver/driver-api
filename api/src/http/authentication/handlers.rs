use axum::extract::State;
use plannify_driver_api_core::domain::driver::{entities::{CreateDriverRequest, CreateDriverResponse}, port::DriverService};

use crate::{AppState, http::common::{api_error::{ApiError, ErrorBody}, response::Response, validator::ValidatedJson}};

#[utoipa::path(
    post,
    path = "/authentication/signup",
    tag = "authentication",
    request_body = CreateDriverRequest,
    responses(
        (status = 201, description = "Driver signed up successfully", body = CreateDriverResponse),
        (status = 400, description = "Email domain denylisted", body = ErrorBody),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn signup(
    State(state): State<AppState>,
    ValidatedJson(request): ValidatedJson<CreateDriverRequest>,
) -> Result<Response<CreateDriverResponse>, ApiError> {
    let driver = state.service.create_driver(request, state.config.check_content.email_domain_denylist).await?;

    Ok(Response::created(CreateDriverResponse {
        message: format!("Driver '{}' signed up successfully", driver.firstname),
        access_token: "access_token".to_string(),
    }))
}
