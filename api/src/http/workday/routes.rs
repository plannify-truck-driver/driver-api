use utoipa_axum::{router::OpenApiRouter, routes};

use crate::http::{
    common::app_state::AppState,
    workday::handlers::{
        __path_create_workday, __path_delete_workday, __path_delete_workday_garbage,
        __path_get_all_workday_garbage, __path_get_all_workdays_month,
        __path_get_all_workdays_period, __path_get_workday_documents,
        __path_get_workday_documents_by_year, __path_update_workday, create_workday,
        delete_workday, delete_workday_garbage, get_all_workday_garbage, get_all_workdays_month,
        get_all_workdays_period, get_workday_documents, get_workday_documents_by_year,
        update_workday,
    },
};

pub fn workday_routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(get_all_workdays_month))
        .routes(routes!(get_all_workdays_period))
        .routes(routes!(create_workday))
        .routes(routes!(update_workday))
        .routes(routes!(delete_workday))
        .routes(routes!(get_all_workday_garbage))
        .routes(routes!(delete_workday_garbage))
        .routes(routes!(get_workday_documents))
        .routes(routes!(get_workday_documents_by_year))
}
