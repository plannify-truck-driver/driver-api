use api::http::common::api_error::ErrorBody;
use axum::http::StatusCode;
use plannify_driver_api_core::domain::mail::entities::{DriverMail, MailStatus};
use serial_test::serial;
use test_context::test_context;

use crate::context;

// IDs définis dans config/test-dataset.sql
const MAIL_USER_A_1: &str = "223e4567-e89b-12d3-a456-426614174000"; // SUCCESS, type 1, sans PJ
const MAIL_USER_A_2: &str = "223e4567-e89b-12d3-a456-426614174001"; // PENDING, type 4, avec PJ
const MAIL_USER_B: &str = "223e4567-e89b-12d3-a456-426614174002"; // appartient à User B

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_mail_unauthorized(ctx: &mut context::TestContext) {
    let res = ctx
        .unauthenticated_router
        .get(&format!("/mails/{}", MAIL_USER_A_1))
        .await;

    res.assert_status(StatusCode::UNAUTHORIZED);

    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "UNAUTHORIZED");
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_mail_not_found(ctx: &mut context::TestContext) {
    let res = ctx
        .authenticated_router
        .get("/mails/00000000-0000-0000-0000-000000000000")
        .await;

    res.assert_status(StatusCode::NOT_FOUND);

    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "MAIL_NOT_FOUND");
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_mail_success_without_attachment(ctx: &mut context::TestContext) {
    let res = ctx
        .authenticated_router
        .get(&format!("/mails/{}", MAIL_USER_A_1))
        .await;

    res.assert_status(StatusCode::OK);

    let body: DriverMail = res.json();
    assert_eq!(body.pk_driver_mail_id.to_string(), MAIL_USER_A_1);
    assert_eq!(body.mail_type.pk_driver_mail_type_id, 1);
    assert_eq!(body.mail_type.label, "ACCOUNT_VERIFICATION");
    assert_eq!(body.status, MailStatus::SUCCESS);
    assert_eq!(body.email_used, "test.user@example.be");
    assert_eq!(body.description, "Account verification email");
    assert!(body.sent_at.is_some());
    assert_eq!(body.attachments.len(), 0);
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_mail_success_with_attachment(ctx: &mut context::TestContext) {
    let res = ctx
        .authenticated_router
        .get(&format!("/mails/{}", MAIL_USER_A_2))
        .await;

    res.assert_status(StatusCode::OK);

    let body: DriverMail = res.json();
    assert_eq!(body.pk_driver_mail_id.to_string(), MAIL_USER_A_2);
    assert_eq!(body.mail_type.pk_driver_mail_type_id, 4);
    assert_eq!(body.mail_type.label, "MONTHLY_REPORTS");
    assert_eq!(body.status, MailStatus::PENDING);
    assert!(body.sent_at.is_none());
    assert_eq!(body.attachments.len(), 1);
    assert_eq!(body.attachments[0].file_name, "attachment-1.pdf");
    assert_eq!(
        body.attachments[0].pk_driver_mail_attachment_id.to_string(),
        "423e4567-e89b-12d3-a456-426614174000"
    );
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_mail_cross_user_isolation(ctx: &mut context::TestContext) {
    // User B essaie d'accéder au mail de User A → 404
    let other_router = ctx.create_authenticated_router_with_different_user().await;

    let res = other_router.get(&format!("/mails/{}", MAIL_USER_A_1)).await;

    res.assert_status(StatusCode::NOT_FOUND);

    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "MAIL_NOT_FOUND");
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_mail_cross_user_can_access_own(ctx: &mut context::TestContext) {
    // User B peut accéder à son propre mail
    let other_router = ctx.create_authenticated_router_with_different_user().await;

    let res = other_router.get(&format!("/mails/{}", MAIL_USER_B)).await;

    res.assert_status(StatusCode::OK);

    let body: DriverMail = res.json();
    assert_eq!(body.pk_driver_mail_id.to_string(), MAIL_USER_B);
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_mail_cache(ctx: &mut context::TestContext) {
    let res1 = ctx
        .authenticated_router
        .get(&format!("/mails/{}", MAIL_USER_A_1))
        .await;
    res1.assert_status(StatusCode::OK);
    let body1: DriverMail = res1.json();

    let res2 = ctx
        .authenticated_router
        .get(&format!("/mails/{}", MAIL_USER_A_1))
        .await;
    res2.assert_status(StatusCode::OK);
    let body2: DriverMail = res2.json();

    assert_eq!(body1.pk_driver_mail_id, body2.pk_driver_mail_id);
    assert_eq!(body1.status, body2.status);
}
