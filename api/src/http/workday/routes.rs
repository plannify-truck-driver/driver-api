use utoipa_axum::{router::OpenApiRouter, routes};

use crate::http::{
    common::app_state::AppState,
    workday::handlers::{__path_get_all_month, get_all_month},
};

pub fn workday_routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(get_all_month))
}
