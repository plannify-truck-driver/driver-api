use utoipa_axum::{router::OpenApiRouter, routes};

use crate::http::{
    authentication::handlers::{__path_login, __path_signup, login, signup},
    common::app_state::AppState,
};

pub fn authentication_routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(signup))
        .routes(routes!(login))
}
