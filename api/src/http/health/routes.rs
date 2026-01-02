use axum::{Router, routing::get};

use crate::http::{common::app_state::AppState, health::handlers::health_check};

pub fn health_routes() -> Router<AppState> {
    Router::new().route("/health", get(health_check))
}
