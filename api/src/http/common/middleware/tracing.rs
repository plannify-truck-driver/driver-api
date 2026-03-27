use axum::{extract::Request, middleware::Next, response::Response};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use std::time::Instant;
use tracing::info;

use crate::http::common::middleware::auth::entities::AccessClaims;

pub async fn tracing_middleware(
    request: Request,
    next: Next,
    jwt_secret: Option<String>,
) -> Response {
    let start = Instant::now();
    let method = request.method().clone();
    let uri = request.uri().clone();
    let path = uri.path().to_string();

    let driver_id =
        extract_driver_id(&request, jwt_secret).unwrap_or_else(|| "unknown".to_string());

    let response = next.run(request).await;

    let duration = start.elapsed();
    let status = response.status();
    let status_code = status.as_u16();

    info!(
        method = %method,
        route = %path,
        driver_id = %driver_id,
        status = status_code,
        duration_ms = duration.as_millis(),
        "API Response"
    );

    response
}

fn extract_token_from_request(request: &Request) -> Option<String> {
    // Check Authorization header first
    if let Some(token) = request
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|v| v.trim().to_string())
    {
        return Some(token);
    }

    // Fall back to access_token cookie
    let cookie_header = request
        .headers()
        .get("Cookie")
        .and_then(|v| v.to_str().ok())?;

    cookie_header
        .split(';')
        .map(|s| s.trim())
        .find(|s| s.starts_with("access_token="))
        .and_then(|s| s.strip_prefix("access_token="))
        .map(|v| v.to_string())
}

fn extract_driver_id(request: &Request, jwt_secret: Option<String>) -> Option<String> {
    let jwt_secret = jwt_secret.as_ref()?;
    let token = extract_token_from_request(request)?;

    let token_data = decode::<AccessClaims>(
        &token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )
    .ok()?;

    Some(token_data.claims.sub.to_string())
}
