use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    AppState,
    http::update::handlers::{__path_get_all_updates_by_version, get_all_updates_by_version},
};

pub fn update_routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(get_all_updates_by_version))
}
