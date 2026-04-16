use api::http::common::api_error::ErrorBody;
use axum::http::StatusCode;
use plannify_driver_api_core::domain::workday::port::WorkdayDatabaseRepository;
use serde_json::json;
use serial_test::serial;
use test_context::test_context;

use crate::context;

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_workday_document_years_unauthorized(ctx: &mut context::TestContext) {
    let res = ctx
        .unauthenticated_router
        .get("/workdays/documents/year")
        .await;

    res.assert_status(StatusCode::UNAUTHORIZED);

    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "UNAUTHORIZED");
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_workday_document_years_success(ctx: &mut context::TestContext) {
    let res = ctx
        .authenticated_router
        .get("/workdays/documents/year")
        .await;

    res.assert_status(StatusCode::OK);

    let body: Vec<i32> = res.json();

    assert_eq!(
        body.len(),
        4,
        "there should be exactly four years available"
    );
    assert!(
        body.contains(&2025),
        "2025 should be in the available years"
    );
    assert!(
        body.contains(&2026),
        "2026 should be in the available years"
    );
    assert!(
        body.contains(&2027),
        "2027 should be in the available years"
    );
    assert!(
        body.contains(&2031),
        "2031 should be in the available years"
    );
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_workday_document_years_update_cache_success(ctx: &mut context::TestContext) {
    let res = ctx
        .authenticated_router
        .get("/workdays/documents/year")
        .await;

    res.assert_status(StatusCode::OK);

    let body: Vec<i32> = res.json();

    assert_eq!(
        body.len(),
        4,
        "there should be exactly four years available"
    );
    assert!(
        body.contains(&2025),
        "2025 should be in the available years"
    );
    assert!(
        body.contains(&2026),
        "2026 should be in the available years"
    );
    assert!(
        body.contains(&2027),
        "2027 should be in the available years"
    );
    assert!(
        body.contains(&2031),
        "2031 should be in the available years"
    );

    ctx.authenticated_router
        .post("/workdays")
        .json(&json!({
            "date": "2030-01-01",
            "start_time": "08:00:00",
            "end_time": null,
            "rest_time": "00:00:00",
            "overnight_rest": false
        }))
        .await;

    let res = ctx
        .authenticated_router
        .get("/workdays/documents/year")
        .await;

    res.assert_status(StatusCode::OK);

    let body: Vec<i32> = res.json();

    assert_eq!(
        body.len(),
        5,
        "there should be exactly five years available"
    );
    assert!(
        body.contains(&2025),
        "2025 should be in the available years"
    );
    assert!(
        body.contains(&2026),
        "2026 should be in the available years"
    );
    assert!(
        body.contains(&2027),
        "2027 should be in the available years"
    );
    assert!(
        body.contains(&2030),
        "2030 should be in the available years"
    );
    assert!(
        body.contains(&2031),
        "2031 should be in the available years"
    );

    ctx.repositories
        .workday_database_repository
        .delete_workday(
            ctx.authenticated_user_id,
            chrono::NaiveDate::from_ymd_opt(2030, 1, 1).unwrap(),
        )
        .await
        .ok();
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_workday_document_years_cross_user_isolation(ctx: &mut context::TestContext) {
    // User A sees [2025, 2026, 2027, 2031].
    // User B has only one non-garbage workday (2026-01-01) and no documents,
    // so they should see only [2026].
    let other_router = ctx.create_authenticated_router_with_different_user().await;

    let res = other_router.get("/workdays/documents/year").await;

    res.assert_status(StatusCode::OK);
    let body: Vec<i32> = res.json();

    assert_eq!(body.len(), 1, "User B should see exactly 1 year");
    assert!(body.contains(&2026), "User B's only year should be 2026");
    assert!(!body.contains(&2025), "User B must not see User A's 2025");
    assert!(!body.contains(&2027), "User B must not see User A's 2027");
    assert!(!body.contains(&2031), "User B must not see User A's 2031");
}
