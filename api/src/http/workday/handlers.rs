use axum::{
    Extension,
    body::Body,
    extract::{Path, State},
    http::{HeaderValue, StatusCode, header},
    response::IntoResponse,
};
use chrono::NaiveDate;
use plannify_driver_api_core::domain::workday::{
    entities::{
        CreateWorkdayRequest, GetWorkdaysByMonthParams, GetWorkdaysByPeriodParams,
        UpdateWorkdayRequest, Workday, WorkdayGarbage,
    },
    port::WorkdayService,
};

use crate::{
    ApiError, AppState,
    http::common::{
        api_error::ErrorBody,
        middleware::auth::entities::UserIdentity,
        response::{PaginatedResponse, Response},
        validator::{ValidatedJson, ValidatedQuery},
    },
};

#[utoipa::path(
    get,
    path = "/workdays/month",
    tag = "workdays",
    description = "Retrieve workdays for a specific month and year",
    params(GetWorkdaysByMonthParams),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Monthly workdays retrieved successfully", body = Vec<Workday>),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn get_all_workdays_month(
    ValidatedQuery(query): ValidatedQuery<GetWorkdaysByMonthParams>,
    State(state): State<AppState>,
    Extension(user_identity): Extension<UserIdentity>,
) -> Result<Response<Vec<Workday>>, ApiError> {
    let workdays = state
        .service
        .get_workdays_by_month(user_identity.user_id, query.month, query.year)
        .await?;

    Ok(Response::ok(workdays))
}

#[utoipa::path(
    get,
    path = "/workdays",
    tag = "workdays",
    description = "Retrieve workdays for a specific period with pagination",
    params(GetWorkdaysByPeriodParams),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Period workdays retrieved successfully", body = PaginatedResponse<Workday>),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn get_all_workdays_period(
    ValidatedQuery(query): ValidatedQuery<GetWorkdaysByPeriodParams>,
    State(state): State<AppState>,
    Extension(user_identity): Extension<UserIdentity>,
) -> Result<Response<PaginatedResponse<Workday>>, ApiError> {
    let (workdays, total_count) = state
        .service
        .get_workdays_by_period(
            user_identity.user_id,
            query.from,
            query.to,
            query.page,
            query.limit,
        )
        .await?;

    let response_workdays: PaginatedResponse<Workday> = PaginatedResponse {
        data: workdays,
        total: total_count,
        page: query.page,
    };

    Ok(Response::ok(response_workdays))
}

#[utoipa::path(
    post,
    path = "/workdays",
    tag = "workdays",
    description = "Create a new workday",
    request_body = CreateWorkdayRequest,
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 201, description = "Workday created successfully", body = Workday),
        (status = 409, description = "Workday already exists", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn create_workday(
    State(state): State<AppState>,
    Extension(user_identity): Extension<UserIdentity>,
    ValidatedJson(request): ValidatedJson<CreateWorkdayRequest>,
) -> Result<Response<Workday>, ApiError> {
    let workday = state
        .service
        .create_workday(user_identity.user_id, request)
        .await?;

    Ok(Response::created(workday.to_workday()))
}

#[utoipa::path(
    put,
    path = "/workdays",
    tag = "workdays",
    description = "Update an existing workday",
    request_body = UpdateWorkdayRequest,
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Workday updated successfully", body = Workday),
        (status = 404, description = "Workday not found", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn update_workday(
    State(state): State<AppState>,
    Extension(user_identity): Extension<UserIdentity>,
    ValidatedJson(request): ValidatedJson<UpdateWorkdayRequest>,
) -> Result<Response<Workday>, ApiError> {
    let workday = state
        .service
        .update_workday(user_identity.user_id, request)
        .await?;

    Ok(Response::ok(workday.to_workday()))
}

#[utoipa::path(
    delete,
    path = "/workdays/{date}",
    tag = "workdays",
    description = "Delete a workday by date",
    params(
        ("date" = NaiveDate, Path, description = "The date of the workday to delete")
    ),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Workday deleted successfully"),
        (status = 404, description = "Workday not found", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn delete_workday(
    State(state): State<AppState>,
    Extension(user_identity): Extension<UserIdentity>,
    Path(date): Path<NaiveDate>,
) -> Result<Response<()>, ApiError> {
    state
        .service
        .create_workday_garbage(user_identity.user_id, date)
        .await?;

    Ok(Response::ok(()))
}

#[utoipa::path(
    get,
    path = "/workdays/garbage",
    tag = "workdays/garbage",
    description = "Retrieve all workdays garbage",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Workdays garbage retrieved successfully", body = Vec<WorkdayGarbage>),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn get_all_workday_garbage(
    State(state): State<AppState>,
    Extension(user_identity): Extension<UserIdentity>,
) -> Result<Response<Vec<WorkdayGarbage>>, ApiError> {
    let workdays = state
        .service
        .get_workdays_garbage(user_identity.user_id)
        .await?;
    let response_workdays: Vec<WorkdayGarbage> =
        workdays.iter().map(|w| w.to_workday_garbage()).collect();

    Ok(Response::ok(response_workdays))
}

#[utoipa::path(
    delete,
    path = "/workdays/garbage/{date}",
    tag = "workdays/garbage",
    description = "Delete a workday garbage and restore the workday",
    params(
        ("date" = NaiveDate, Path, description = "The date of the workday garbage to delete")
    ),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Workday garbage deleted successfully"),
        (status = 404, description = "Workday garbage not found", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn delete_workday_garbage(
    State(state): State<AppState>,
    Extension(user_identity): Extension<UserIdentity>,
    Path(date): Path<NaiveDate>,
) -> Result<Response<()>, ApiError> {
    state
        .service
        .delete_workday_garbage(user_identity.user_id, date)
        .await?;

    Ok(Response::ok(()))
}

#[utoipa::path(
    get,
    path = "/workdays/documents/year",
    tag = "workdays/documents",
    description = "Retrieve years with workday documents",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Workday document years retrieved successfully", body = Vec<i32>),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn get_workday_documents(
    State(state): State<AppState>,
    Extension(user_identity): Extension<UserIdentity>,
) -> Result<Response<Vec<i32>>, ApiError> {
    let documents = state
        .service
        .get_workday_documents(user_identity.user_id)
        .await?;

    Ok(Response::ok(documents))
}

#[utoipa::path(
    get,
    path = "/workdays/documents/{year}",
    tag = "workdays/documents",
    description = "Retrieve months with workday documents for a specific year",
    params(
        ("year" = i32, Path, description = "The year of the workday documents to retrieve")
    ),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Workday document months retrieved successfully", body = Vec<i32>),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn get_workday_documents_by_year(
    State(state): State<AppState>,
    Extension(user_identity): Extension<UserIdentity>,
    Path(year): Path<i32>,
) -> Result<Response<Vec<i32>>, ApiError> {
    let documents = state
        .service
        .get_workday_documents_by_year(user_identity.user_id, year)
        .await?;

    Ok(Response::ok(documents))
}

#[utoipa::path(
    get,
    path = "/workdays/documents/{year}/{month}",
    tag = "workdays/documents",
    description = "Download monthly workday report as PDF",
    params(
        ("year" = i32, Path, description = "Year"),
        ("month" = i32, Path, description = "Month (1-12)")
    ),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "PDF file", body = [u8]),
        (status = 404, description = "No document for this month", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn get_workday_document_by_month(
    State(state): State<AppState>,
    Extension(user_identity): Extension<UserIdentity>,
    Path((year, month)): Path<(i32, i32)>,
) -> Result<impl IntoResponse, ApiError> {
    if !(1..=12).contains(&month) {
        return Err(ApiError::BadRequest {
            error_code: "INVALID_MONTH".to_string(),
            content: None,
        });
    }

    let pdf = state
        .service
        .get_workday_document_by_month(user_identity.user_id, month, year)
        .await?;

    let pdf = pdf.ok_or_else(|| ApiError::NotFound {
        error_code: "DOCUMENT_NOT_FOUND".to_string(),
    })?;

    let filename = format!("workdays-{}-{:02}.pdf", year, month);
    let mut headers = axum::http::HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/pdf"),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::try_from(format!("attachment; filename=\"{}\"", filename))
            .unwrap_or(HeaderValue::from_static("attachment")),
    );

    Ok((StatusCode::OK, (headers, Body::from(pdf.to_vec()))))
}
