use axum::{
    Extension,
    body::Body,
    extract::{Path, State},
    http::{StatusCode, header, HeaderValue},
    response::IntoResponse,
};
use plannify_driver_api_core::domain::mail::{
    entities::{
        DriverMail, DriverMailPreference, DriverMailType, GetMailsParams, UpdateMailPreferenceRequest,
    },
    port::MailService,
};
use uuid::Uuid;

use crate::{
    ApiError, AppState,
    http::common::{
        api_error::ErrorBody,
        middleware::auth::entities::UserIdentity,
        response::{PaginatedResponse, Response},
        validator::{ValidatedJson, ValidatedQuery},
    },
};

#[tracing::instrument(
    name = "get_mails",
    skip_all,
    fields(
        user_id = %user_identity.user_id,
        page = %query.page,
        limit = %query.limit,
        count = tracing::field::Empty,
    )
)]
#[utoipa::path(
    get,
    path = "/mails",
    tag = "mails",
    description = "Retrieve driver mails with pagination",
    params(GetMailsParams),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Mails retrieved successfully", body = PaginatedResponse<DriverMail>),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn get_mails(
    ValidatedQuery(query): ValidatedQuery<GetMailsParams>,
    State(state): State<AppState>,
    Extension(user_identity): Extension<UserIdentity>,
) -> Result<Response<PaginatedResponse<DriverMail>>, ApiError> {
    let (mails, total) = state
        .service
        .get_mails(user_identity.user_id, query.page, query.limit)
        .await?;

    tracing::Span::current().record("count", mails.len());

    Ok(Response::ok(PaginatedResponse {
        data: mails,
        total,
        page: query.page,
    }))
}

#[tracing::instrument(
    name = "get_mail_types",
    skip_all,
    fields(
        user_id = %user_identity.user_id,
        count = tracing::field::Empty,
    )
)]
#[utoipa::path(
    get,
    path = "/mails/types",
    tag = "mails",
    description = "Retrieve all mail types",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Mail types retrieved successfully", body = Vec<DriverMailType>),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn get_mail_types(
    State(state): State<AppState>,
    Extension(user_identity): Extension<UserIdentity>,
) -> Result<Response<Vec<DriverMailType>>, ApiError> {
    let types = state.service.get_mail_types().await?;

    tracing::Span::current().record("count", types.len());

    Ok(Response::ok(types))
}

#[tracing::instrument(
    name = "get_mail_preferences",
    skip_all,
    fields(
        user_id = %user_identity.user_id,
        count = tracing::field::Empty,
    )
)]
#[utoipa::path(
    get,
    path = "/mails/preferences",
    tag = "mails/preferences",
    description = "Retrieve driver mail preferences",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Mail preferences retrieved successfully", body = Vec<DriverMailPreference>),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn get_mail_preferences(
    State(state): State<AppState>,
    Extension(user_identity): Extension<UserIdentity>,
) -> Result<Response<Vec<DriverMailPreference>>, ApiError> {
    let preferences = state
        .service
        .get_mail_preferences(user_identity.user_id)
        .await?;

    tracing::Span::current().record("count", preferences.len());

    Ok(Response::ok(preferences))
}

#[tracing::instrument(
    name = "update_mail_preference",
    skip_all,
    fields(
        user_id = %user_identity.user_id,
        mail_type_id = %mail_type_id,
        is_enabled = %request.is_enabled,
    )
)]
#[utoipa::path(
    put,
    path = "/mails/preferences/{mail_type_id}",
    tag = "mails/preferences",
    description = "Update a driver mail preference",
    params(
        ("mail_type_id" = i32, Path, description = "The mail type ID to update the preference for")
    ),
    request_body = UpdateMailPreferenceRequest,
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Mail preference updated successfully", body = DriverMailPreference),
        (status = 400, description = "Mail preference is not editable", body = ErrorBody),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 404, description = "Mail type not found", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn update_mail_preference(
    Path(mail_type_id): Path<i32>,
    State(state): State<AppState>,
    Extension(user_identity): Extension<UserIdentity>,
    ValidatedJson(request): ValidatedJson<UpdateMailPreferenceRequest>,
) -> Result<Response<DriverMailPreference>, ApiError> {
    let preference = state
        .service
        .update_mail_preference(user_identity.user_id, mail_type_id, request.is_enabled)
        .await?;

    Ok(Response::ok(preference))
}

#[tracing::instrument(
    name = "get_mail",
    skip_all,
    fields(
        user_id = %user_identity.user_id,
        mail_id = %mail_id,
    )
)]
#[utoipa::path(
    get,
    path = "/mails/{mail_id}",
    tag = "mails",
    description = "Retrieve a single driver mail by ID",
    params(
        ("mail_id" = Uuid, Path, description = "The mail ID")
    ),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Mail retrieved successfully", body = DriverMail),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 404, description = "Mail not found", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn get_mail(
    Path(mail_id): Path<Uuid>,
    State(state): State<AppState>,
    Extension(user_identity): Extension<UserIdentity>,
) -> Result<Response<DriverMail>, ApiError> {
    let mail = state
        .service
        .get_mail(user_identity.user_id, mail_id)
        .await?;

    Ok(Response::ok(mail))
}

#[tracing::instrument(
    name = "get_mail_attachment",
    skip_all,
    fields(
        user_id = %user_identity.user_id,
        attachment_id = %attachment_id,
    )
)]
#[utoipa::path(
    get,
    path = "/mails/attachments/{attachment_id}",
    tag = "mails",
    description = "Download a mail attachment file",
    params(
        ("attachment_id" = Uuid, Path, description = "The attachment ID")
    ),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "File downloaded successfully", body = [u8]),
        (status = 401, description = "Unauthorized", body = ErrorBody),
        (status = 404, description = "Attachment not found", body = ErrorBody),
        (status = 500, description = "Internal server error", body = ErrorBody)
    )
)]
pub async fn get_mail_attachment(
    Path(attachment_id): Path<Uuid>,
    State(state): State<AppState>,
    Extension(user_identity): Extension<UserIdentity>,
) -> Result<impl IntoResponse, ApiError> {
    let (bytes, file_name) = state
        .service
        .download_mail_attachment(user_identity.user_id, attachment_id)
        .await?;

    let mut headers = axum::http::HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/octet-stream"),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::try_from(format!("attachment; filename=\"{}\"", file_name))
            .unwrap_or(HeaderValue::from_static("attachment")),
    );

    Ok((StatusCode::OK, (headers, Body::from(bytes.to_vec()))))
}
