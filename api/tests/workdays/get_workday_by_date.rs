use api::http::common::api_error::ErrorBody;
use axum::http::StatusCode;
use plannify_driver_api_core::domain::workday::entities::Workday;
use serial_test::serial;
use test_context::test_context;

use crate::{context, workdays::verify_workday_content};

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_workday_by_date_unauthorized(ctx: &mut context::TestContext) {
    let res = ctx.unauthenticated_router.get("/workdays/2026-01-01").await;

    res.assert_status(StatusCode::UNAUTHORIZED);

    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "UNAUTHORIZED");
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_workday_by_date_success(ctx: &mut context::TestContext) {
    let res = ctx.authenticated_router.get("/workdays/2026-01-01").await;

    res.assert_status(StatusCode::OK);

    let body: Workday = res.json();

    let expected_workday = Workday {
        date: chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        start_time: chrono::NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
        end_time: chrono::NaiveTime::from_hms_opt(17, 45, 0),
        rest_time: chrono::NaiveTime::from_hms_opt(1, 0, 0).unwrap(),
        overnight_rest: true,
    };

    verify_workday_content(body, expected_workday);
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_other_workday_by_date_error(ctx: &mut context::TestContext) {
    let res = ctx.authenticated_router.get("/workdays/2026-01-02").await;

    res.assert_status(StatusCode::NOT_FOUND);
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_workday_by_date_already_in_garbage(ctx: &mut context::TestContext) {
    let res = ctx.authenticated_router.get("/workdays/2024-01-01").await;

    res.assert_status(StatusCode::NOT_FOUND);
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_workday_by_date_cross_user_isolation(ctx: &mut context::TestContext) {
    // 2027-01-01 belongs to User A. User B must not be able to read it.
    let other_router = ctx.create_authenticated_router_with_different_user().await;

    let res = other_router.get("/workdays/2027-01-01").await;

    res.assert_status(StatusCode::NOT_FOUND);
    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "WORKDAY_NOT_FOUND");
}
