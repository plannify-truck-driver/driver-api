use utoipa_axum::{router::OpenApiRouter, routes};

use crate::http::{
    common::app_state::AppState,
    drivers::handlers::{__path_signin_driver, signin_driver},
};

pub fn driver_routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(signin_driver))
}
