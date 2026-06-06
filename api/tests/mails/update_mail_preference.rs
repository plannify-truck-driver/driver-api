use api::http::common::api_error::ErrorBody;
use axum::http::StatusCode;
use plannify_driver_api_core::domain::mail::entities::DriverMailPreference;
use serde_json::json;
use serial_test::serial;
use test_context::test_context;

use crate::context;

// Type 1 (ACCOUNT_VERIFICATION) : not editable
// Type 4 (MONTHLY_REPORTS) : editable
const EDITABLE_TYPE_ID: i32 = 4;
const NON_EDITABLE_TYPE_ID: i32 = 1;

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_update_mail_preference_unauthorized(ctx: &mut context::TestContext) {
    let res = ctx
        .unauthenticated_router
        .put(&format!("/mails/preferences/{}", EDITABLE_TYPE_ID))
        .json(&json!({ "is_enabled": true }))
        .await;

    res.assert_status(StatusCode::UNAUTHORIZED);

    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "UNAUTHORIZED");
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_update_mail_preference_type_not_found(ctx: &mut context::TestContext) {
    let res = ctx
        .authenticated_router
        .put("/mails/preferences/9999")
        .json(&json!({ "is_enabled": true }))
        .await;

    res.assert_status(StatusCode::NOT_FOUND);

    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "MAIL_TYPE_NOT_FOUND");
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_update_mail_preference_not_editable(ctx: &mut context::TestContext) {
    let res = ctx
        .authenticated_router
        .put(&format!("/mails/preferences/{}", NON_EDITABLE_TYPE_ID))
        .json(&json!({ "is_enabled": true }))
        .await;

    res.assert_status(StatusCode::BAD_REQUEST);

    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "MAIL_PREFERENCE_NOT_EDITABLE");
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_update_mail_preference_enable_success(ctx: &mut context::TestContext) {
    let res = ctx
        .authenticated_router
        .put(&format!("/mails/preferences/{}", EDITABLE_TYPE_ID))
        .json(&json!({ "is_enabled": true }))
        .await;

    res.assert_status(StatusCode::OK);

    let body: DriverMailPreference = res.json();
    assert_eq!(body.mail_type_id, EDITABLE_TYPE_ID);
    assert!(body.is_enabled);
    assert!(body.is_editable);
    assert_eq!(body.label, "MONTHLY_REPORTS");

    // Cleanup : remettre à disabled
    ctx.authenticated_router
        .put(&format!("/mails/preferences/{}", EDITABLE_TYPE_ID))
        .json(&json!({ "is_enabled": false }))
        .await
        .assert_status(StatusCode::OK);
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_update_mail_preference_disable_success(ctx: &mut context::TestContext) {
    // Setup : activate the preference first
    ctx.authenticated_router
        .put(&format!("/mails/preferences/{}", EDITABLE_TYPE_ID))
        .json(&json!({ "is_enabled": true }))
        .await
        .assert_status(StatusCode::OK);

    // Deactivate the preference
    let res = ctx
        .authenticated_router
        .put(&format!("/mails/preferences/{}", EDITABLE_TYPE_ID))
        .json(&json!({ "is_enabled": false }))
        .await;

    res.assert_status(StatusCode::OK);

    let body: DriverMailPreference = res.json();
    assert_eq!(body.mail_type_id, EDITABLE_TYPE_ID);
    assert!(!body.is_enabled);
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_update_mail_preference_invalidates_cache(ctx: &mut context::TestContext) {
    // Put preference in cache
    let res = ctx.authenticated_router.get("/mails/preferences").await;
    res.assert_status(StatusCode::OK);
    let before: Vec<DriverMailPreference> = res.json();
    let pref_before = before
        .iter()
        .find(|p| p.mail_type_id == EDITABLE_TYPE_ID)
        .unwrap();
    assert!(!pref_before.is_enabled);

    // Update preference (must invalidate the cache)
    ctx.authenticated_router
        .put(&format!("/mails/preferences/{}", EDITABLE_TYPE_ID))
        .json(&json!({ "is_enabled": true }))
        .await
        .assert_status(StatusCode::OK);

    // The cache being invalidated, the next read must return the new value
    let res = ctx.authenticated_router.get("/mails/preferences").await;
    res.assert_status(StatusCode::OK);
    let after: Vec<DriverMailPreference> = res.json();
    let pref_after = after
        .iter()
        .find(|p| p.mail_type_id == EDITABLE_TYPE_ID)
        .unwrap();
    assert!(
        pref_after.is_enabled,
        "the cache must be invalidated after the update"
    );

    // Cleanup
    ctx.authenticated_router
        .put(&format!("/mails/preferences/{}", EDITABLE_TYPE_ID))
        .json(&json!({ "is_enabled": false }))
        .await
        .assert_status(StatusCode::OK);
}
