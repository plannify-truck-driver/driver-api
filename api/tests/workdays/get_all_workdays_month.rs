use api::http::common::api_error::ErrorBody;
use axum::http::StatusCode;
use plannify_driver_api_core::domain::workday::entities::Workday;
use serial_test::serial;
use test_context::test_context;

use crate::{context, workdays::verify_workday_content};

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_all_workdays_month_unauthorized(ctx: &mut context::TestContext) {
    let res = ctx
        .unauthenticated_router
        .get("/workdays/month?month=1&year=2026")
        .await;

    res.assert_status(StatusCode::UNAUTHORIZED);

    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "UNAUTHORIZED");
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_all_workdays_month_success(ctx: &mut context::TestContext) {
    let res = ctx
        .authenticated_router
        .get("/workdays/month?month=1&year=2026")
        .await;

    res.assert_status(StatusCode::OK);

    let body: Vec<Workday> = res.json();
    assert_eq!(
        body.len(),
        2,
        "response array must contain exactly two workdays"
    );

    let expected_workday1 = Workday {
        date: chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        start_time: chrono::NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
        end_time: chrono::NaiveTime::from_hms_opt(17, 45, 0),
        rest_time: chrono::NaiveTime::from_hms_opt(1, 0, 0).unwrap(),
        overnight_rest: true,
    };
    let workday_json1 = &body[0];
    verify_workday_content(*workday_json1, expected_workday1);

    let expected_workday2 = Workday {
        date: chrono::NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
        start_time: chrono::NaiveTime::from_hms_opt(7, 40, 0).unwrap(),
        end_time: None,
        rest_time: chrono::NaiveTime::from_hms_opt(0, 30, 0).unwrap(),
        overnight_rest: false,
    };
    let workday_json2 = &body[1];
    verify_workday_content(*workday_json2, expected_workday2);
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_all_workdays_month_with_no_parameters(ctx: &mut context::TestContext) {
    let res1 = ctx.authenticated_router.get("/workdays/month").await;

    res1.assert_status(StatusCode::BAD_REQUEST);

    let body1: ErrorBody = res1.json();
    assert_eq!(body1.error_code, "MISSING_ATTRIBUTE");

    let res2 = ctx
        .authenticated_router
        .get("/workdays/month?month=1")
        .await;

    res2.assert_status(StatusCode::BAD_REQUEST);

    let body2: ErrorBody = res2.json();
    assert_eq!(body2.error_code, "MISSING_ATTRIBUTE");

    let res3 = ctx
        .authenticated_router
        .get("/workdays/month?year=2026")
        .await;

    res3.assert_status(StatusCode::BAD_REQUEST);

    let body3: ErrorBody = res3.json();
    assert_eq!(body3.error_code, "MISSING_ATTRIBUTE");
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_all_workdays_month_with_wrong_parameters(ctx: &mut context::TestContext) {
    let res1 = ctx
        .authenticated_router
        .get("/workdays/month?month=0&year=2026")
        .await;

    res1.assert_status(StatusCode::BAD_REQUEST);

    let body1: ErrorBody = res1.json();
    assert_eq!(body1.error_code, "QUERY_VALIDATION");

    let res2 = ctx
        .authenticated_router
        .get("/workdays/month?month=13&year=2026")
        .await;

    res2.assert_status(StatusCode::BAD_REQUEST);

    let body2: ErrorBody = res2.json();
    assert_eq!(body2.error_code, "QUERY_VALIDATION");

    let res3 = ctx
        .authenticated_router
        .get("/workdays/month?month=1&year=1800")
        .await;

    res3.assert_status(StatusCode::BAD_REQUEST);

    let body3: ErrorBody = res3.json();
    assert_eq!(body3.error_code, "QUERY_VALIDATION");

    let res4 = ctx
        .authenticated_router
        .get("/workdays/month?month=1&year=2200")
        .await;

    res4.assert_status(StatusCode::BAD_REQUEST);

    let body4: ErrorBody = res4.json();
    assert_eq!(body4.error_code, "QUERY_VALIDATION");
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_all_workdays_month_cross_user_isolation(ctx: &mut context::TestContext) {
    // User B has no workdays in January 2027 — the result must be empty.
    let other_router = ctx.create_authenticated_router_with_different_user().await;

    let res = other_router
        .get("/workdays/month?month=1&year=2027")
        .await;

    res.assert_status(StatusCode::OK);
    let body: Vec<Workday> = res.json();
    assert!(
        body.is_empty(),
        "User B should have no workdays in January 2027, got {}",
        body.len()
    );
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_all_workdays_month_cache_invalidation(ctx: &mut context::TestContext) {
    // Warm up the monthly cache for January 2026 (2 workdays: 2026-01-01 and 2026-01-31)
    let initial_res = ctx
        .authenticated_router
        .get("/workdays/month?month=1&year=2026")
        .await;
    initial_res.assert_status(StatusCode::OK);
    let initial_body: Vec<Workday> = initial_res.json();
    assert_eq!(
        initial_body.len(),
        2,
        "initial state: 2 workdays in January 2026"
    );

    // Soft-delete 2026-01-31 — the service must also invalidate the monthly cache
    ctx.authenticated_router
        .delete("/workdays/2026-01-31")
        .await
        .assert_status(StatusCode::OK);

    // The cache must be invalidated: only 2026-01-01 is visible now
    let updated_res = ctx
        .authenticated_router
        .get("/workdays/month?month=1&year=2026")
        .await;
    updated_res.assert_status(StatusCode::OK);
    let updated_body: Vec<Workday> = updated_res.json();
    assert_eq!(
        updated_body.len(),
        1,
        "after soft-deleting 2026-01-31, only 1 workday should remain visible in January 2026"
    );
    assert_eq!(
        updated_body[0].date,
        chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()
    );

    // Cleanup: restore 2026-01-31 (also clears the monthly cache via the service)
    ctx.authenticated_router
        .delete("/workdays/garbage/2026-01-31")
        .await;
}
