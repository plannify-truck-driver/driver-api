use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts},
};
use axum_extra::extract::CookieJar;

use tracing::error;

use crate::http::common::{api_error::ApiError, middleware::auth::entities::TokenValidator};
pub mod entities;

pub struct AuthMiddleware;

impl<AuthValidator> FromRequestParts<AuthValidator> for AuthMiddleware
where
    AuthValidator: Send + Sync + TokenValidator,
{
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AuthValidator,
    ) -> Result<Self, Self::Rejection> {
        let cookie_jar = CookieJar::from_request_parts(parts, state)
            .await
            .map_err(|e| {
                error!("Failed to extract cookies: {:?}", e);
                ApiError::Unauthorized {
                    error_code: "UNAUTHORIZED".to_string(),
                }
            })?;

        // try to get token from cookies
        let auth_cookie = cookie_jar.get("access_token");

        let token = if let Some(auth_cookie) = auth_cookie {
            auth_cookie.value().to_string()
        } else {
            // extract token from Authorization header
            parts
                .headers
                .get(header::AUTHORIZATION)
                .and_then(|auth_header| auth_header.to_str().ok())
                .and_then(|auth_str| auth_str.strip_prefix("Bearer "))
                .ok_or_else(|| ApiError::Unauthorized {
                    error_code: "UNAUTHORIZED".to_string(),
                })?
                .to_string()
        };

        let user_identity = state.validate_token(&token)?;

        // add auth state to request
        parts.extensions.insert(user_identity);
        Ok(Self)
    }
}

pub struct AuthRefreshMiddleware;

impl<AuthValidator> FromRequestParts<AuthValidator> for AuthRefreshMiddleware
where
    AuthValidator: Send + Sync + TokenValidator,
{
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AuthValidator,
    ) -> Result<Self, Self::Rejection> {
        let cookie_jar = CookieJar::from_request_parts(parts, state)
            .await
            .map_err(|e| {
                error!("Failed to extract cookies: {:?}", e);
                ApiError::Unauthorized {
                    error_code: "UNAUTHORIZED".to_string(),
                }
            })?;

        // try to get token from cookies
        let auth_cookie = cookie_jar.get("refresh_token");
        let token = auth_cookie
            .ok_or_else(|| {
                error!("Refresh token not found in cookies");
                ApiError::Unauthorized {
                error_code: "UNAUTHORIZED".to_string(),
            }})?
            .value()
            .to_string();

        let user_identity = state.validate_refresh_token(&token)?;

        parts.extensions.insert(user_identity);
        Ok(Self)
    }
}
