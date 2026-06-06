use api::http::common::api_error::ErrorBody;
use api::http::common::response::PaginatedResponse;
use axum::http::StatusCode;
use plannify_driver_api_core::domain::mail::entities::DriverMail;
use serial_test::serial;
use test_context::test_context;
use uuid::Uuid;

use crate::context;

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
    assert_eq!(body.data.len(), 10);
    assert_eq!(body.page, 1);
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_mails_pagination(ctx: &mut context::TestContext) {
    let res = ctx.authenticated_router.get("/mails?page=1&limit=1").await;
    res.assert_status(StatusCode::OK);
    let body: PaginatedResponse<DriverMail> = res.json();
    assert_eq!(body.data.len(), 1);

    let res = ctx.authenticated_router.get("/mails?page=2&limit=1").await;
    res.assert_status(StatusCode::OK);
    let body: PaginatedResponse<DriverMail> = res.json();
    assert_eq!(body.data.len(), 1);

    let res = ctx.authenticated_router.get("/mails?page=3&limit=100").await;
    res.assert_status(StatusCode::OK);
    let body: PaginatedResponse<DriverMail> = res.json();
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
