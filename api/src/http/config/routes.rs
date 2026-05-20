use utoipa_axum::{router::OpenApiRouter, routes};

use crate::http::{
    common::app_state::AppState,
    config::handlers::{__path_get_config, get_config},
};

pub fn config_routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(get_config))
}
