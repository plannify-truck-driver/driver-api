use axum::{Extension, extract::State, http::header::SET_COOKIE, response::AppendHeaders};

type AuthResponse = Result<
    (
        AppendHeaders<[(axum::http::HeaderName, String); 2]>,
        Response<CreateDriverResponse>,
    ),
    ApiError,
>;

type RefreshResponse = Result<
    (
        AppendHeaders<[(axum::http::HeaderName, String); 1]>,
        Response<()>,
    ),
    ApiError,
>;

use plannify_driver_api_core::domain::{
    driver::{
        entities::{
            ConfirmPasswordResetRequest, CreateDriverRequest, CreateDriverResponse, DriverRow,
            LoginDriverRequest, RequestPasswordResetRequest, VerifyDriverAccountRequest,
        },
        port::DriverService,
    },
    mail::port::MailService,
};
use plannify_driver_api_core::infrastructure::driver::repositories::error::DriverError;

use tracing::error;

use crate::{
    AppState,
    http::common::{
        api_error::{ApiError, ErrorBody},
        middleware::auth::entities::{TokenValidator, UserIdentity},
        response::Response,
        validator::ValidatedJson,
    },
};

#[tracing::instrument(
    name = "signup",
    skip_all,
    fields(
        firstname = %request.firstname,
        lastname = %request.lastname,
        email = %request.email,
    )
)]
#[utoipa::path(
    post,
    path = "/authentication/signup",
    tag = "authentication",
    security(),
    request_body = CreateDriverRequest,
    responses(
        (status = 201, description = "Driver signed up successfully", body = CreateDriverResponse),
        (status = 400, description = "Email domain denylisted", body = ErrorBody),
        (status = 403, description = "Account verification mail preference is disabled", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn signup(
    State(state): State<AppState>,
    ValidatedJson(request): ValidatedJson<CreateDriverRequest>,
) -> AuthResponse {
    let driver = state
        .service
        .create_driver(request, state.config.check_content.email_domain_denylist)
        .await?;

    state.service.send_creation_email(driver.clone()).await?;

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
        Response::created(CreateDriverResponse { access_token }),
    ))
}

#[tracing::instrument(
    name = "login",
    skip_all,
    fields(
        email = %request.email,
    )
)]
#[utoipa::path(
    post,
    path = "/authentication/login",
    tag = "authentication",
    security(),
    request_body = LoginDriverRequest,
    responses(
        (status = 200, description = "Driver logged in successfully", body = CreateDriverResponse),
        (status = 401, description = "Invalid credentials", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn login(
    State(state): State<AppState>,
    ValidatedJson(request): ValidatedJson<LoginDriverRequest>,
) -> AuthResponse {
    let driver = state.service.login_driver(request).await?;

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
    name = "verify_driver_account",
    skip_all,
    fields(
        driver_id = %request.driver_id,
    )
)]
#[utoipa::path(
    post,
    path = "/authentication/token/verify-account",
    tag = "authentication",
    security(),
    request_body = VerifyDriverAccountRequest,
    responses(
        (status = 200, description = "Driver account verified successfully", body = CreateDriverResponse),
        (status = 401, description = "Invalid credentials", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn verify_driver_account(
    State(state): State<AppState>,
    ValidatedJson(request): ValidatedJson<VerifyDriverAccountRequest>,
) -> AuthResponse {
    let driver = state
        .service
        .verify_driver_account(request.driver_id, request.token)
        .await?;

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
    name = "refresh_token",
    skip_all,
    fields(
        driver_id = %user_identity.user_id,
    )
)]
#[utoipa::path(
    get,
    path = "/authentication/refresh",
    tag = "authentication",
    security(),
    responses(
        (status = 200, description = "Driver auth refreshed successfully", body = CreateDriverResponse),
        (status = 401, description = "Invalid token", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn refresh_token(
    State(state): State<AppState>,
    Extension(user_identity): Extension<UserIdentity>,
) -> AuthResponse {
    let driver = state
        .service
        .get_driver_for_refresh(user_identity.user_id)
        .await?;

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
    name = "request_password_reset",
    skip_all,
    fields(
        email = %request.email,
    )
)]
#[utoipa::path(
    post,
    path = "/authentication/reset-password",
    tag = "authentication",
    security(),
    request_body = RequestPasswordResetRequest,
    responses(
        (status = 200, description = "Password reset email sent successfully"),
        (status = 403, description = "Password reset mail preference is disabled", body = ErrorBody),
        (status = 404, description = "Driver not found", body = ErrorBody),
        (status = 409, description = "Reset password token already exists", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn request_password_reset(
    State(state): State<AppState>,
    ValidatedJson(request): ValidatedJson<RequestPasswordResetRequest>,
) -> Result<Response<()>, ApiError> {
    let driver = state.service.request_password_reset(request.email).await?;
    state.service.send_reset_password_email(driver).await?;
    Ok(Response::ok(()))
}

#[tracing::instrument(
    name = "confirm_password_reset",
    skip_all,
    fields(
        driver_id = %request.driver_id,
    )
)]
#[utoipa::path(
    post,
    path = "/authentication/confirm-reset-password",
    tag = "authentication",
    security(),
    request_body = ConfirmPasswordResetRequest,
    responses(
        (status = 200, description = "Password updated successfully"),
        (status = 400, description = "Invalid or expired reset token", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn confirm_password_reset(
    State(state): State<AppState>,
    ValidatedJson(request): ValidatedJson<ConfirmPasswordResetRequest>,
) -> Result<Response<()>, ApiError> {
    let driver = state
        .service
        .confirm_password_reset(request.driver_id, request.token, request.password)
        .await?;
    state
        .service
        .send_password_change_notification(driver)
        .await?;
    Ok(Response::ok(()))
}

#[tracing::instrument(name = "delete_refresh_token", skip_all)]
#[utoipa::path(
    delete,
    path = "/authentication/refresh",
    tag = "authentication",
    responses(
        (status = 200, description = "Driver auth deleted successfully"),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn delete_refresh_token(State(state): State<AppState>) -> RefreshResponse {
    let refresh_token_cookie = state
        .service
        .delete_refresh_token(state.config.common.frontend_url.as_str())
        .await?;

    Ok((
        AppendHeaders([(SET_COOKIE, refresh_token_cookie)]),
        Response::ok(()),
    ))
}
