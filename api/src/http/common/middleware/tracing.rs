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

fn extract_driver_id(request: &Request, jwt_secret: Option<String>) -> Option<String> {
    let jwt_secret = jwt_secret.as_ref()?;

    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())?;

    let token = auth_header.strip_prefix("Bearer ")?.trim();

    // Try to decode the token without verification (just to get the claims)
    // In production, you might want to properly validate with your secret key
    let token_data = decode::<AccessClaims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )
    .ok()?;

    Some(token_data.claims.sub.to_string())
}
