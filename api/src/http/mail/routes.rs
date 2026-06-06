use utoipa_axum::{router::OpenApiRouter, routes};

use crate::http::{
    common::app_state::AppState,
    mail::handlers::{
        __path_get_mail, __path_get_mail_attachment, __path_get_mail_preferences,
        __path_get_mail_types, __path_get_mails, __path_update_mail_preference, get_mail,
        get_mail_attachment, get_mail_preferences, get_mail_types, get_mails,
        update_mail_preference,
    },
};

pub fn mail_routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(get_mails))
        .routes(routes!(get_mail_types))
        .routes(routes!(get_mail_preferences))
        .routes(routes!(update_mail_preference))
        .routes(routes!(get_mail))
        .routes(routes!(get_mail_attachment))
}
