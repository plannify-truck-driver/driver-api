use api::http::common::api_error::ErrorBody;
use axum::http::StatusCode;
use plannify_driver_api_core::domain::mail::entities::DriverMailType;
use serial_test::serial;
use test_context::test_context;

use crate::context;

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_mail_types_unauthorized(ctx: &mut context::TestContext) {
    let res = ctx.unauthenticated_router.get("/mails/types").await;

    res.assert_status(StatusCode::UNAUTHORIZED);

    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "UNAUTHORIZED");
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_mail_types_success(ctx: &mut context::TestContext) {
    let res = ctx.authenticated_router.get("/mails/types").await;

    res.assert_status(StatusCode::OK);

    let body: Vec<DriverMailType> = res.json();
    assert_eq!(body.len(), 4, "4 types de mail doivent exister");

    assert_eq!(body[0].pk_driver_mail_type_id, 1);
    assert_eq!(body[0].label, "ACCOUNT_VERIFICATION");
    assert!(!body[0].is_editable);

    assert_eq!(body[1].pk_driver_mail_type_id, 2);
    assert_eq!(body[1].label, "PASSWORD_RESET");
    assert!(!body[1].is_editable);

    assert_eq!(body[2].pk_driver_mail_type_id, 3);
    assert_eq!(body[2].label, "ACCOUNT_CHANGEMENT");
    assert!(!body[2].is_editable);

    assert_eq!(body[3].pk_driver_mail_type_id, 4);
    assert_eq!(body[3].label, "MONTHLY_REPORTS");
    assert!(body[3].is_editable);
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_mail_types_cache(ctx: &mut context::TestContext) {
    let res1 = ctx.authenticated_router.get("/mails/types").await;
    res1.assert_status(StatusCode::OK);
    let body1: Vec<DriverMailType> = res1.json();

    let res2 = ctx.authenticated_router.get("/mails/types").await;
    res2.assert_status(StatusCode::OK);
    let body2: Vec<DriverMailType> = res2.json();

    assert_eq!(body1.len(), body2.len());
    assert_eq!(
        body1[0].pk_driver_mail_type_id,
        body2[0].pk_driver_mail_type_id
    );
}
