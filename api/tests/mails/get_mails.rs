use api::http::common::api_error::ErrorBody;
use api::http::common::response::PaginatedResponse;
use axum::http::StatusCode;
use plannify_driver_api_core::domain::mail::entities::DriverMail;
use serial_test::serial;
use test_context::test_context;
use uuid::Uuid;

use crate::context;

// IDs définis dans config/test-dataset.sql
const MAIL_USER_A_1: &str = "223e4567-e89b-12d3-a456-426614174000"; // SUCCESS, type 1
const MAIL_USER_A_2: &str = "223e4567-e89b-12d3-a456-426614174001"; // PENDING, type 4

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_mails_unauthorized(ctx: &mut context::TestContext) {
    let res = ctx
        .unauthenticated_router
        .get("/mails?page=1&limit=10")
        .await;

    res.assert_status(StatusCode::UNAUTHORIZED);

    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "UNAUTHORIZED");
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_mails_missing_params(ctx: &mut context::TestContext) {
    let res = ctx.authenticated_router.get("/mails").await;
    res.assert_status(StatusCode::BAD_REQUEST);
    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "MISSING_ATTRIBUTE");

    let res = ctx.authenticated_router.get("/mails?page=1").await;
    res.assert_status(StatusCode::BAD_REQUEST);
    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "MISSING_ATTRIBUTE");

    let res = ctx.authenticated_router.get("/mails?limit=10").await;
    res.assert_status(StatusCode::BAD_REQUEST);
    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "MISSING_ATTRIBUTE");
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_mails_invalid_params(ctx: &mut context::TestContext) {
    let res = ctx.authenticated_router.get("/mails?page=0&limit=10").await;
    res.assert_status(StatusCode::BAD_REQUEST);
    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "QUERY_VALIDATION");

    let res = ctx.authenticated_router.get("/mails?page=1&limit=0").await;
    res.assert_status(StatusCode::BAD_REQUEST);
    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "QUERY_VALIDATION");

    let res = ctx
        .authenticated_router
        .get("/mails?page=1&limit=101")
        .await;
    res.assert_status(StatusCode::BAD_REQUEST);
    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "QUERY_VALIDATION");
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_mails_success(ctx: &mut context::TestContext) {
    let res = ctx.authenticated_router.get("/mails?page=1&limit=10").await;

    res.assert_status(StatusCode::OK);

    let body: PaginatedResponse<DriverMail> = res.json();
    assert_eq!(body.total, 2, "User A doit avoir 2 mails");
    assert_eq!(body.data.len(), 2);
    assert_eq!(body.page, 1);

    // Triés par created_at DESC : mail 2 (2026-02) avant mail 1 (2026-01)
    let mail_ids: Vec<String> = body
        .data
        .iter()
        .map(|m| m.pk_driver_mail_id.to_string())
        .collect();
    assert_eq!(mail_ids[0], MAIL_USER_A_2);
    assert_eq!(mail_ids[1], MAIL_USER_A_1);

    // Mail 2 a une pièce jointe
    assert_eq!(body.data[0].attachments.len(), 1);
    assert_eq!(body.data[0].attachments[0].file_name, "attachment-1.pdf");

    // Mail 1 n'a pas de pièce jointe
    assert_eq!(body.data[1].attachments.len(), 0);
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_mails_pagination(ctx: &mut context::TestContext) {
    let res = ctx.authenticated_router.get("/mails?page=1&limit=1").await;
    res.assert_status(StatusCode::OK);
    let body: PaginatedResponse<DriverMail> = res.json();
    assert_eq!(body.total, 2);
    assert_eq!(body.data.len(), 1);
    assert_eq!(body.data[0].pk_driver_mail_id.to_string(), MAIL_USER_A_2);

    let res = ctx.authenticated_router.get("/mails?page=2&limit=1").await;
    res.assert_status(StatusCode::OK);
    let body: PaginatedResponse<DriverMail> = res.json();
    assert_eq!(body.total, 2);
    assert_eq!(body.data.len(), 1);
    assert_eq!(body.data[0].pk_driver_mail_id.to_string(), MAIL_USER_A_1);

    let res = ctx.authenticated_router.get("/mails?page=3&limit=1").await;
    res.assert_status(StatusCode::OK);
    let body: PaginatedResponse<DriverMail> = res.json();
    assert_eq!(body.total, 2);
    assert_eq!(body.data.len(), 0);
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_mails_cross_user_isolation(ctx: &mut context::TestContext) {
    let other_router = ctx.create_authenticated_router_with_different_user().await;

    let res = other_router.get("/mails?page=1&limit=10").await;
    res.assert_status(StatusCode::OK);

    let body: PaginatedResponse<DriverMail> = res.json();
    assert_eq!(
        body.total, 1,
        "User B doit avoir uniquement son propre mail"
    );
    assert_eq!(
        body.data[0].pk_driver_mail_id,
        Uuid::parse_str("223e4567-e89b-12d3-a456-426614174002").unwrap()
    );
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_mails_cache(ctx: &mut context::TestContext) {
    let res1 = ctx.authenticated_router.get("/mails?page=1&limit=10").await;
    res1.assert_status(StatusCode::OK);
    let body1: PaginatedResponse<DriverMail> = res1.json();

    let res2 = ctx.authenticated_router.get("/mails?page=1&limit=10").await;
    res2.assert_status(StatusCode::OK);
    let body2: PaginatedResponse<DriverMail> = res2.json();

    assert_eq!(body1.total, body2.total);
    assert_eq!(body1.data.len(), body2.data.len());
    assert_eq!(
        body1.data[0].pk_driver_mail_id,
        body2.data[0].pk_driver_mail_id
    );
}
