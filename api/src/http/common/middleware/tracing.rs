use axum::{extract::Request, middleware::Next, response::Response};
use std::time::Instant;
use tracing::info;
use uuid::Uuid;

pub async fn tracing_middleware(request: Request, next: Next) -> Response {
    let trace_id = Uuid::new_v4();
    let start = Instant::now();
    let method = request.method().clone();
    let uri = request.uri().clone();
    let path = uri.path().to_string();

    info!(
        trace_id = %trace_id,
        method = %method,
        route = %path,
        "API Request"
    );

    let response = next.run(request).await;

    let duration = start.elapsed();
    let status = response.status();
    let status_code = status.as_u16();

    info!(
        trace_id = %trace_id,
        method = %method,
        route = %path,
        status = status_code,
        duration_ms = duration.as_millis(),
        "API Response"
    );

    response
}
