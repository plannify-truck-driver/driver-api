use utoipa_axum::{router::OpenApiRouter, routes};

use crate::http::{
    authentication::handlers::{
        __path_delete_refresh_token, __path_login, __path_refresh_token, __path_signup,
        __path_verify_driver_account, delete_refresh_token, login, refresh_token, signup,
        verify_driver_account,
    },
    common::app_state::AppState,
};

pub fn authentication_routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(signup))
        .routes(routes!(login))
        .routes(routes!(verify_driver_account))
}

pub fn refresh_cookie_routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(refresh_token))
}

pub fn unauthenticated_routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(delete_refresh_token))
}
