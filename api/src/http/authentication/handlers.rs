use axum::{Extension, extract::State, http::header::SET_COOKIE, response::AppendHeaders};
use plannify_driver_api_core::domain::{
    driver::{
        entities::{
            CreateDriverRequest, CreateDriverResponse, DriverRow, LoginDriverRequest,
            VerifyDriverAccountRequest,
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

#[utoipa::path(
    post,
    path = "/authentication/signup",
    tag = "authentication",
    security(),
    request_body = CreateDriverRequest,
    responses(
        (status = 201, description = "Driver signed up successfully", body = CreateDriverResponse),
        (status = 400, description = "Email domain denylisted", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn signup(
    State(state): State<AppState>,
    ValidatedJson(request): ValidatedJson<CreateDriverRequest>,
) -> Result<
    (
        AppendHeaders<[(axum::http::HeaderName, String); 1]>,
        Response<CreateDriverResponse>,
    ),
    ApiError,
> {
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

    let (access_token, refresh_token_cookie) = state
        .service
        .generate_tokens(
            driver,
            create_tokens_fn,
            state.config.jwt.refresh_ttl,
            state.config.common.frontend_url.as_str(),
        )
        .await?;

    let headers = [(SET_COOKIE, refresh_token_cookie)];

    Ok((
        AppendHeaders(headers),
        Response::created(CreateDriverResponse { access_token }),
    ))
}

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
) -> Result<
    (
        AppendHeaders<[(axum::http::HeaderName, String); 1]>,
        Response<CreateDriverResponse>,
    ),
    ApiError,
> {
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

    let (access_token, refresh_token_cookie) = state
        .service
        .generate_tokens(
            driver,
            create_tokens_fn,
            state.config.jwt.refresh_ttl,
            state.config.common.frontend_url.as_str(),
        )
        .await?;

    let headers = [(SET_COOKIE, refresh_token_cookie)];

    Ok((
        AppendHeaders(headers),
        Response::ok(CreateDriverResponse { access_token }),
    ))
}

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
) -> Result<
    (
        AppendHeaders<[(axum::http::HeaderName, String); 1]>,
        Response<CreateDriverResponse>,
    ),
    ApiError,
> {
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

    let (access_token, refresh_token_cookie) = state
        .service
        .generate_tokens(
            driver,
            create_tokens_fn,
            state.config.jwt.refresh_ttl,
            state.config.common.frontend_url.as_str(),
        )
        .await?;

    let headers = [(SET_COOKIE, refresh_token_cookie)];

    Ok((
        AppendHeaders(headers),
        Response::ok(CreateDriverResponse { access_token }),
    ))
}

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
) -> Result<
    (
        AppendHeaders<[(axum::http::HeaderName, String); 1]>,
        Response<CreateDriverResponse>,
    ),
    ApiError,
> {
    let driver = state
        .service
        .get_driver_by_id(user_identity.user_id)
        .await?;

    let driver = driver.ok_or_else(|| ApiError::Unauthorized {
        error_code: "INVALID_TOKEN".to_string(),
    })?;

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

    let (access_token, refresh_token_cookie) = state
        .service
        .generate_tokens(
            driver,
            create_tokens_fn,
            state.config.jwt.refresh_ttl,
            state.config.common.frontend_url.as_str(),
        )
        .await?;

    let headers = [(SET_COOKIE, refresh_token_cookie)];

    Ok((
        AppendHeaders(headers),
        Response::ok(CreateDriverResponse { access_token }),
    ))
}

#[utoipa::path(
    delete,
    path = "/authentication/refresh",
    tag = "authentication",
    responses(
        (status = 200, description = "Driver auth deleted successfully"),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn delete_refresh_token(
    State(state): State<AppState>,
) -> Result<
    (
        AppendHeaders<[(axum::http::HeaderName, String); 1]>,
        Response<()>,
    ),
    ApiError,
> {
    let refresh_token_cookie = state
        .service
        .delete_refresh_token(state.config.common.frontend_url.as_str())
        .await?;

    Ok((
        AppendHeaders([(SET_COOKIE, refresh_token_cookie)]),
        Response::ok(())
    ))
}