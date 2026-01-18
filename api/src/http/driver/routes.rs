use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    AppState,
    http::driver::handlers::{
        __path_delete_rest_periods, __path_get_all_rest_periods, __path_set_rest_periods,
        delete_rest_periods, get_all_rest_periods, set_rest_periods,
    },
};

pub fn driver_routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(get_all_rest_periods))
        .routes(routes!(set_rest_periods))
        .routes(routes!(delete_rest_periods))
}
