use axum::{extract::{MatchedPath, Request}, middleware::Next, response::Response};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use std::time::Instant;
use tracing::Instrument;

use crate::http::common::middleware::auth::entities::AccessClaims;

pub async fn tracing_middleware(
    request: Request,
    next: Next,
    jwt_secret: Option<String>,
) -> Response {
    let method = request.method().clone();
    let route = request
        .extensions()
        .get::<MatchedPath>()
        .map(MatchedPath::as_str)
        .unwrap_or_else(|| request.uri().path())
        .to_string();
    let driver_id =
        extract_driver_id(&request, jwt_secret).unwrap_or_else(|| "unknown".to_string());

    let span = tracing::info_span!(
        "http_request",
        "otel.name" = %route,
        method = %method,
        route = %route,
        driver_id = %driver_id,
        status = tracing::field::Empty,
        duration_ms = tracing::field::Empty,
    );

    let start = Instant::now();
    let response = next.run(request).instrument(span.clone()).await;

    let status = response.status().as_u16();
    let duration_ms = start.elapsed().as_millis();

    span.record("status", status);
    span.record("duration_ms", duration_ms);

    span.in_scope(|| {
        tracing::info!(status, duration_ms, "API Response");
    });

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
