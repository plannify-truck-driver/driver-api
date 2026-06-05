use axum::{Extension, extract::State, http::header::SET_COOKIE, response::AppendHeaders};
use plannify_driver_api_core::domain::{
    driver::{
        entities::{
            CreateDriverResponse, CreateDriverRestPeriodsRequest, DriverRestPeriod, DriverRow, GetDriverResponse, UpdateDriverRequest
        }, port::DriverService
    },
    mail::port::MailService,
};
use plannify_driver_api_core::infrastructure::driver::repositories::error::DriverError;
use tracing::error;

use crate::{
    ApiError, AppState,
    http::common::{
        api_error::ErrorBody,
        middleware::auth::entities::{TokenValidator, UserIdentity},
        response::Response,
        validator::ValidatedJson,
    },
};

type UpdateDriverResponse = Result<
    (
        AppendHeaders<[(axum::http::HeaderName, String); 2]>,
        Response<CreateDriverResponse>,
    ),
    ApiError,
>;

#[tracing::instrument(
    name = "get_all_rest_periods",
    skip_all,
    fields(
        user_id = %user_identity.user_id,
        result.count = tracing::field::Empty,
    )
)]
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
        (status = 401, description = "Unauthorized", body = ErrorBody),
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

    tracing::Span::current().record("result.total", rest_periods.len());

    Ok(Response::ok(rest_periods))
}

#[tracing::instrument(
    name = "set_rest_periods",
    skip_all,
    fields(
        user_id = %user_identity.user_id,
        result.count = tracing::field::Empty,
    )
)]
#[utoipa::path(
    post,
    path = "/rest-periods",
    tag = "driver/rest-periods",
    description = "Set driver rest periods",
    request_body = CreateDriverRestPeriodsRequest,
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 201, description = "Driver rest periods set successfully"),
        (status = 400, description = "Invalid rest period", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn set_rest_periods(
    State(state): State<AppState>,
    Extension(user_identity): Extension<UserIdentity>,
    ValidatedJson(request): ValidatedJson<CreateDriverRestPeriodsRequest>,
) -> Result<Response<()>, ApiError> {
    let rest_periods_count = request.rest_periods.len();

    state
        .service
        .set_driver_rest_periods(user_identity.user_id, request.rest_periods)
        .await?;

    tracing::Span::current().record("result.count", rest_periods_count);

    Ok(Response::created(()))
}

#[tracing::instrument(
    name = "delete_rest_periods",
    skip_all,
    fields(
        user_id = %user_identity.user_id,
    )
)]
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
        (status = 401, description = "Unauthorized", body = ErrorBody),
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

#[tracing::instrument(
    name = "get_driver_info",
    skip_all,
)]
#[utoipa::path(
    get,
    path = "/me",
    tag = "driver",
    description = "Get driver personal information.",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Driver information retrieved successfully", body = GetDriverResponse),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn get_driver_info(
    State(state): State<AppState>,
    Extension(user_identity): Extension<UserIdentity>,
) -> Result<Response<GetDriverResponse>, ApiError> {
    let driver = state
        .service
        .get_driver_by_id(user_identity.user_id)
        .await?
        .ok_or(ApiError::from(DriverError::DriverNotFound))?;

    Ok(Response::ok(GetDriverResponse {
        pk_driver_id: driver.pk_driver_id,
        firstname: driver.firstname,
        lastname: driver.lastname,
        gender: driver.gender,
        email: driver.email,
        phone_number: driver.phone_number,
        is_searchable: driver.is_searchable,
        allow_request_professional_agreement: driver.allow_request_professional_agreement,
        created_at: driver.created_at,
        verified_at: driver.verified_at,
        last_login_at: driver.last_login_at,
        deactivated_at: driver.deactivated_at,
    }))
}

#[tracing::instrument(
    name = "update_driver_info",
    skip_all,
    fields(
        user_id = %user_identity.user_id,
        email_changed = tracing::field::Empty,
    )
)]
#[utoipa::path(
    patch,
    path = "/me",
    tag = "driver",
    description = "Update driver personal information. Only provided fields are updated.",
    request_body = UpdateDriverRequest,
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Driver information updated successfully", body = CreateDriverResponse),
        (status = 400, description = "Validation error", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 404, description = "Driver not found", body = ErrorBody),
        (status = 409, description = "Email already taken", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn update_driver_info(
    State(state): State<AppState>,
    Extension(user_identity): Extension<UserIdentity>,
    ValidatedJson(request): ValidatedJson<UpdateDriverRequest>,
) -> UpdateDriverResponse {
    let (driver, email_changed) = state
        .service
        .update_driver_info(
            user_identity.user_id,
            request,
            state.config.check_content.email_domain_denylist,
        )
        .await?;

    tracing::Span::current().record("email_changed", email_changed);

    if email_changed {
        state
            .service
            .send_email_change_verification(driver.clone())
            .await?;
    }

    let auth_validator = &state.auth_validator;
    let create_tokens_fn = |driver: &DriverRow| -> Result<(String, String), DriverError> {
        auth_validator.create_tokens(driver).map_err(|e| {
            error!(
                "Failed to create tokens for driver {}: {:?}",
                driver.pk_driver_id, e
            );
            DriverError::Internal
        })
    };

    let (access_token, access_token_cookie, refresh_token_cookie) = state
        .service
        .generate_tokens(
            driver,
            create_tokens_fn,
            state.config.jwt.access_ttl,
            state.config.jwt.refresh_ttl,
            state.config.common.frontend_url.as_str(),
        )
        .await?;

    let headers = [
        (SET_COOKIE, access_token_cookie),
        (SET_COOKIE, refresh_token_cookie),
    ];

    Ok((
        AppendHeaders(headers),
        Response::ok(CreateDriverResponse { access_token }),
    ))
}

#[tracing::instrument(
    name = "deactivate_driver",
    skip_all,
    fields(user_id = %user_identity.user_id)
)]
#[utoipa::path(
    post,
    path = "/me/deactivate",
    tag = "driver",
    description = "Deactivate the authenticated driver's account.",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Account deactivated successfully"),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 409, description = "Account is already deactivated", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn deactivate_driver(
    State(state): State<AppState>,
    Extension(user_identity): Extension<UserIdentity>,
) -> Result<Response<()>, ApiError> {
    let driver = state
        .service
        .deactivate_driver(user_identity.user_id)
        .await?;

    state.service.send_deactivation_email(driver).await?;

    Ok(Response::ok(()))
}

#[tracing::instrument(
    name = "reactivate_driver",
    skip_all,
    fields(user_id = %user_identity.user_id)
)]
#[utoipa::path(
    post,
    path = "/me/reactivate",
    tag = "driver",
    description = "Reactivate the authenticated driver's account.",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Account reactivated successfully"),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 409, description = "Account is not deactivated", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn reactivate_driver(
    State(state): State<AppState>,
    Extension(user_identity): Extension<UserIdentity>,
) -> Result<Response<()>, ApiError> {
    let driver = state
        .service
        .reactivate_driver(user_identity.user_id)
        .await?;

    state.service.send_reactivation_email(driver).await?;

    Ok(Response::ok(()))
}
