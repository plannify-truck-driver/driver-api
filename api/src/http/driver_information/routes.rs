use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    AppState,
    http::driver_information::handlers::{__path_get_driver_informations, get_driver_informations},
};

pub fn driver_information_routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(get_driver_informations))
}
