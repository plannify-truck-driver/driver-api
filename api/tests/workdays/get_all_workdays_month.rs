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
