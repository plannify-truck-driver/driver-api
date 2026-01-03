use utoipa_axum::{router::OpenApiRouter, routes};

use crate::http::{
    authentication::handlers::{__path_signup, signup},
    common::app_state::AppState,
};

pub fn authentication_routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(signup))
}
