use api::http::common::api_error::ErrorBody;
use axum::http::StatusCode;
use plannify_driver_api_core::domain::mail::entities::DriverMailPreference;
use serial_test::serial;
use test_context::test_context;

use crate::context;

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_mail_preferences_unauthorized(ctx: &mut context::TestContext) {
    let res = ctx.unauthenticated_router.get("/mails/preferences").await;

    res.assert_status(StatusCode::UNAUTHORIZED);

    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "UNAUTHORIZED");
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_mail_preferences_success(ctx: &mut context::TestContext) {
    let res = ctx.authenticated_router.get("/mails/preferences").await;

    res.assert_status(StatusCode::OK);

    let body: Vec<DriverMailPreference> = res.json();
    assert_eq!(body.len(), 4, "4 preferences must exist (one per type)");

    // mail_preferences = 0 in dataset → all preferences should be disabled
    assert!(
        body.iter().all(|p| !p.is_enabled),
        "all preferences should be disabled (bitmask = 0)"
    );

    // only MONTHLY_REPORTS (id=4) is editable
    let editable: Vec<&DriverMailPreference> = body.iter().filter(|p| p.is_editable).collect();
    assert_eq!(editable.len(), 1);
    assert_eq!(editable[0].mail_type_id, 4);
    assert_eq!(editable[0].label, "MONTHLY_REPORTS");
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_mail_preferences_cache(ctx: &mut context::TestContext) {
    let res1 = ctx.authenticated_router.get("/mails/preferences").await;
    res1.assert_status(StatusCode::OK);
    let body1: Vec<DriverMailPreference> = res1.json();

    let res2 = ctx.authenticated_router.get("/mails/preferences").await;
    res2.assert_status(StatusCode::OK);
    let body2: Vec<DriverMailPreference> = res2.json();

    assert_eq!(body1.len(), body2.len());
    assert_eq!(body1[0].mail_type_id, body2[0].mail_type_id);
    assert_eq!(body1[0].is_enabled, body2[0].is_enabled);
}
