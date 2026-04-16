use api::http::common::api_error::ErrorBody;
use axum::http::StatusCode;
use plannify_driver_api_core::domain::workday::entities::WorkdayGarbage;
use serial_test::serial;
use test_context::test_context;

use crate::context;

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_all_workday_garbage_unauthorized(ctx: &mut context::TestContext) {
    let res = ctx.unauthenticated_router.get("/workdays/garbage").await;

    res.assert_status(StatusCode::UNAUTHORIZED);

    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "UNAUTHORIZED");
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_all_workday_garbage_success(ctx: &mut context::TestContext) {
    let res = ctx.authenticated_router.get("/workdays/garbage").await;

    res.assert_status(StatusCode::OK);

    let body: Vec<WorkdayGarbage> = res.json();
    assert_eq!(body.len(), 2, "there should be exactly two workday garbage");
    assert_eq!(
        body[0].workday_date,
        chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()
    );
    assert_eq!(
        body[0].created_at,
        chrono::NaiveDateTime::new(
            chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            chrono::NaiveTime::from_hms_opt(8, 45, 0).unwrap()
        )
        .and_utc()
    );
    assert_eq!(
        body[0].scheduled_deletion_date,
        chrono::NaiveDate::from_ymd_opt(2024, 2, 1).unwrap()
    );

    assert_eq!(
        body[1].workday_date,
        chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap()
    );
    assert_eq!(
        body[1].created_at,
        chrono::NaiveDateTime::new(
            chrono::NaiveDate::from_ymd_opt(2026, 2, 10).unwrap(),
            chrono::NaiveTime::from_hms_opt(11, 30, 0).unwrap()
        )
        .and_utc()
    );
    assert_eq!(
        body[1].scheduled_deletion_date,
        chrono::NaiveDate::from_ymd_opt(2026, 3, 11).unwrap()
    );
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_all_workday_garbage_cross_user_isolation(ctx: &mut context::TestContext) {
    // User B has exactly one garbage entry (2026-01-02).
    // User A's entries (2024-01-01 and 2026-01-15) must NOT appear.
    let other_router = ctx.create_authenticated_router_with_different_user().await;

    let res = other_router.get("/workdays/garbage").await;

    res.assert_status(StatusCode::OK);
    let body: Vec<WorkdayGarbage> = res.json();

    assert_eq!(body.len(), 1, "User B should see exactly one garbage entry");
    assert_eq!(
        body[0].workday_date,
        chrono::NaiveDate::from_ymd_opt(2026, 1, 2).unwrap(),
        "User B's only garbage entry should be 2026-01-02"
    );
}
