use utoipa_axum::{router::OpenApiRouter, routes};

use crate::http::{
    common::app_state::AppState,
    workday::handlers::{
        __path_create_workday, __path_get_all_month, __path_get_all_period, create_workday,
        get_all_month, get_all_period,
    },
};

pub fn workday_routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(get_all_month))
        .routes(routes!(get_all_period))
        .routes(routes!(create_workday))
}
