use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    AppState,
    http::driver::handlers::{
        __path_deactivate_driver, __path_delete_rest_periods, __path_get_all_rest_periods,
        __path_reactivate_driver, __path_set_rest_periods, __path_update_driver_info,
        deactivate_driver, delete_rest_periods, get_all_rest_periods, reactivate_driver,
        set_rest_periods, update_driver_info,
    },
};

pub fn driver_routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(get_all_rest_periods))
        .routes(routes!(set_rest_periods))
        .routes(routes!(delete_rest_periods))
        .routes(routes!(update_driver_info))
        .routes(routes!(deactivate_driver))
        .routes(routes!(reactivate_driver))
}
