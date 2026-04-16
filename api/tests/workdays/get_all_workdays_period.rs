use api::http::common::{api_error::ErrorBody, response::PaginatedResponse};
use axum::http::StatusCode;
use plannify_driver_api_core::domain::workday::entities::Workday;
use serial_test::serial;
use test_context::test_context;

use crate::{context, workdays::verify_workday_content};

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_all_workdays_period_unauthorized(ctx: &mut context::TestContext) {
    let res = ctx
        .unauthenticated_router
        .get("/workdays?from=2026-01-01&to=2026-01-31&page=1&limit=10")
        .await;

    res.assert_status(StatusCode::UNAUTHORIZED);

    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "UNAUTHORIZED");
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_all_workdays_period_success(ctx: &mut context::TestContext) {
    let res = ctx
        .authenticated_router
        .get("/workdays?from=2026-01-01&to=2026-12-31&page=1&limit=10")
        .await;

    res.assert_status(StatusCode::OK);

    let body: PaginatedResponse<Workday> = res.json();
    assert_eq!(body.page, 1, "page number should be 1");
    assert_eq!(body.total, 3, "total workdays should be 3");
    assert_eq!(
        body.data.len(),
        3,
        "response array must contain exactly three workdays"
    );

    let expected_workday1 = Workday {
        date: chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        start_time: chrono::NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
        end_time: chrono::NaiveTime::from_hms_opt(17, 45, 0),
        rest_time: chrono::NaiveTime::from_hms_opt(1, 0, 0).unwrap(),
        overnight_rest: true,
    };
    let workday_json1 = &body.data[0];
    verify_workday_content(*workday_json1, expected_workday1);

    let expected_workday2 = Workday {
        date: chrono::NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
        start_time: chrono::NaiveTime::from_hms_opt(7, 40, 0).unwrap(),
        end_time: None,
        rest_time: chrono::NaiveTime::from_hms_opt(0, 30, 0).unwrap(),
        overnight_rest: false,
    };
    let workday_json2 = &body.data[1];
    verify_workday_content(*workday_json2, expected_workday2);

    let expected_workday3 = Workday {
        date: chrono::NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
        start_time: chrono::NaiveTime::from_hms_opt(8, 40, 0).unwrap(),
        end_time: chrono::NaiveTime::from_hms_opt(17, 21, 0),
        rest_time: chrono::NaiveTime::from_hms_opt(1, 30, 0).unwrap(),
        overnight_rest: false,
    };
    let workday_json3 = &body.data[2];
    verify_workday_content(*workday_json3, expected_workday3);
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_all_workdays_period_with_no_parameters(ctx: &mut context::TestContext) {
    let res1 = ctx.authenticated_router.get("/workdays").await;

    res1.assert_status(StatusCode::BAD_REQUEST);

    let body1: ErrorBody = res1.json();
    assert_eq!(body1.error_code, "MISSING_ATTRIBUTE");

    let res2 = ctx
        .authenticated_router
        .get("/workdays?from=2026-01-01&to=2026-01-31")
        .await;

    res2.assert_status(StatusCode::BAD_REQUEST);

    let body2: ErrorBody = res2.json();
    assert_eq!(body2.error_code, "MISSING_ATTRIBUTE");

    let res3 = ctx
        .authenticated_router
        .get("/workdays?page=1&limit=10")
        .await;

    res3.assert_status(StatusCode::BAD_REQUEST);

    let body3: ErrorBody = res3.json();
    assert_eq!(body3.error_code, "MISSING_ATTRIBUTE");
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_all_workdays_period_with_wrong_parameters(ctx: &mut context::TestContext) {
    let res1 = ctx
        .authenticated_router
        .get("/workdays?from=2026-01-01&to=2026-01-31&page=0&limit=10")
        .await;

    res1.assert_status(StatusCode::BAD_REQUEST);

    let body1: ErrorBody = res1.json();
    assert_eq!(body1.error_code, "QUERY_VALIDATION");

    let res2 = ctx
        .authenticated_router
        .get("/workdays?from=2026-01-01&to=2026-01-31&page=1&limit=0")
        .await;

    res2.assert_status(StatusCode::BAD_REQUEST);

    let body2: ErrorBody = res2.json();
    assert_eq!(body2.error_code, "QUERY_VALIDATION");

    let res3 = ctx
        .authenticated_router
        .get("/workdays?from=2026-01-01&to=2026-01-31&page=1&limit=101")
        .await;

    res3.assert_status(StatusCode::BAD_REQUEST);

    let body3: ErrorBody = res3.json();
    assert_eq!(body3.error_code, "QUERY_VALIDATION");

    let res4 = ctx
        .authenticated_router
        .get("/workdays?from=2026-01&to=2026-01-31&page=1&limit=10")
        .await;

    res4.assert_status(StatusCode::BAD_REQUEST);

    let body4: ErrorBody = res4.json();
    assert_eq!(body4.error_code, "MISSING_ATTRIBUTE");
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_all_workdays_period_cross_user_isolation(ctx: &mut context::TestContext) {
    // User B has no workdays in 2027 — the paginated result must be empty.
    let other_router = ctx.create_authenticated_router_with_different_user().await;

    let res = other_router
        .get("/workdays?from=2027-01-01&to=2027-12-31&page=1&limit=10")
        .await;

    res.assert_status(StatusCode::OK);
    let body: PaginatedResponse<Workday> = res.json();
    assert_eq!(body.total, 0, "User B should have no workdays in 2027");
    assert!(body.data.is_empty());
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_all_workdays_period_single_day(ctx: &mut context::TestContext) {
    // A single-day range (from == to) must return only the workday on that date.
    let res = ctx
        .authenticated_router
        .get("/workdays?from=2026-01-01&to=2026-01-01&page=1&limit=10")
        .await;

    res.assert_status(StatusCode::OK);
    let body: PaginatedResponse<Workday> = res.json();
    assert_eq!(body.total, 1, "exactly one workday must match 2026-01-01");
    assert_eq!(body.data.len(), 1);
    assert_eq!(
        body.data[0].date,
        chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()
    );
}
