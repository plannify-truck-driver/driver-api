use api::http::common::api_error::ErrorBody;
use axum::http::StatusCode;
use plannify_driver_api_core::domain::mail::entities::DriverMailAttachment;
use serial_test::serial;
use test_context::test_context;

use crate::context;

// IDs defined in config/test-dataset.sql
const ATTACHMENT_USER_A: &str = "423e4567-e89b-12d3-a456-426614174000"; // belongs to User A's mail

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_mail_attachment_unauthorized(ctx: &mut context::TestContext) {
    let res = ctx
        .unauthenticated_router
        .get(&format!("/mails/attachments/{}", ATTACHMENT_USER_A))
        .await;

    res.assert_status(StatusCode::UNAUTHORIZED);

    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "UNAUTHORIZED");
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_mail_attachment_not_found(ctx: &mut context::TestContext) {
    let res = ctx
        .authenticated_router
        .get("/mails/attachments/00000000-0000-0000-0000-000000000000")
        .await;

    res.assert_status(StatusCode::NOT_FOUND);

    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "MAIL_ATTACHMENT_NOT_FOUND");
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_mail_attachment_success(ctx: &mut context::TestContext) {
    let res = ctx
        .authenticated_router
        .get(&format!("/mails/attachments/{}", ATTACHMENT_USER_A))
        .await;

    res.assert_status(StatusCode::OK);

    let body: DriverMailAttachment = res.json();
    assert_eq!(
        body.pk_driver_mail_attachment_id.to_string(),
        ATTACHMENT_USER_A
    );
    assert_eq!(body.file_name, "attachment-1.pdf");
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_mail_attachment_cross_user_isolation(ctx: &mut context::TestContext) {
    // User B tries to access User A's attachment → 404
    let other_router = ctx.create_authenticated_router_with_different_user().await;

    let res = other_router
        .get(&format!("/mails/attachments/{}", ATTACHMENT_USER_A))
        .await;

    res.assert_status(StatusCode::NOT_FOUND);

    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "MAIL_ATTACHMENT_NOT_FOUND");
}
