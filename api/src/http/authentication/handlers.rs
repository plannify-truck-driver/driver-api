use axum::{extract::State, http::header::SET_COOKIE, response::AppendHeaders};
use plannify_driver_api_core::domain::driver::{
    entities::{CreateDriverRequest, CreateDriverResponse, DriverRow, LoginDriverRequest},
    port::DriverService,
};
use plannify_driver_api_core::infrastructure::driver::repositories::error::DriverError;

use crate::{
    AppState,
    http::common::{
        api_error::{ApiError, ErrorBody},
        middleware::auth::entities::TokenValidator,
        response::Response,
        validator::ValidatedJson,
    },
};

#[utoipa::path(
    post,
    path = "/authentication/signup",
    tag = "authentication",
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

    let auth_validator = &state.auth_validator;
    let create_tokens_fn = |driver: &DriverRow| -> Result<(String, String), DriverError> {
        auth_validator
            .create_tokens(driver)
            .map_err(|_| DriverError::DatabaseError)
    };

    let (access_token, refresh_token_cookie) = state
        .service
        .generate_tokens(driver, create_tokens_fn, state.config.jwt.refresh_ttl)
        .await?;

    let headers = [(SET_COOKIE, refresh_token_cookie)];

    Ok((
        AppendHeaders(headers),
        Response::created(CreateDriverResponse {
            access_token: access_token,
        }),
    ))
}

#[utoipa::path(
    post,
    path = "/authentication/login",
    tag = "authentication",
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
        auth_validator
            .create_tokens(driver)
            .map_err(|_| DriverError::DatabaseError)
    };

    let (access_token, refresh_token_cookie) = state
        .service
        .generate_tokens(driver, create_tokens_fn, state.config.jwt.refresh_ttl)
        .await?;

    let headers = [(SET_COOKIE, refresh_token_cookie)];

    Ok((
        AppendHeaders(headers),
        Response::ok(CreateDriverResponse {
            access_token: access_token,
        }),
    ))
}
