use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    AppState,
    http::formating::handlers::{
        __path_get_email_formating, __path_get_first_name_formating,
        __path_get_last_name_formating, get_email_formating, get_first_name_formating,
        get_last_name_formating,
    },
};

pub fn formating_routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(get_first_name_formating))
        .routes(routes!(get_last_name_formating))
        .routes(routes!(get_email_formating))
}
