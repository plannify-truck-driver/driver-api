use api::http::common::api_error::ErrorBody;
use axum::http::StatusCode;
use plannify_driver_api_core::domain::workday::{
    entities::{UpdateWorkdayRequest, Workday},
    port::WorkdayDatabaseRepository,
};
use serde_json::json;
use serial_test::serial;
use test_context::test_context;

use crate::{context, workdays::verify_workday_content};

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_update_workday_unauthorized(ctx: &mut context::TestContext) {
    let res = ctx
        .unauthenticated_router
        .put("/workdays")
        .json(&json!({
            "date": "2026-01-01",
            "start_time": "08:00:00",
            "end_time": null,
            "rest_time": "00:00:00",
            "overnight_rest": false
        }))
        .await;

    res.assert_status(StatusCode::UNAUTHORIZED);

    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "UNAUTHORIZED");
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_update_workday_success(ctx: &mut context::TestContext) {
    let res = ctx
        .authenticated_router
        .put("/workdays")
        .json(&json!({
            "date": "2027-01-01",
            "start_time": "08:00:00",
            "end_time": null,
            "rest_time": "00:00:00",
            "overnight_rest": false
        }))
        .await;

    res.assert_status(StatusCode::OK);

    let body: Workday = res.json();

    let expected_workday = Workday {
        date: chrono::NaiveDate::from_ymd_opt(2027, 1, 1).unwrap(),
        start_time: chrono::NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
        end_time: None,
        rest_time: chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
        overnight_rest: false,
    };
    verify_workday_content(body, expected_workday);

    ctx.repositories
        .workday_database_repository
        .update_workday(
            ctx.authenticated_user_id,
            UpdateWorkdayRequest {
                date: chrono::NaiveDate::from_ymd_opt(2027, 1, 1).unwrap(),
                start_time: chrono::NaiveTime::from_hms_opt(8, 14, 0).unwrap(),
                end_time: Some(chrono::NaiveTime::from_hms_opt(13, 31, 0).unwrap()),
                rest_time: chrono::NaiveTime::from_hms_opt(1, 45, 0).unwrap(),
                overnight_rest: true,
            },
        )
        .await
        .unwrap();
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_update_workday_not_found(ctx: &mut context::TestContext) {
    let res = ctx
        .authenticated_router
        .put("/workdays")
        .json(&json!({
            "date": "2028-01-01",
            "start_time": "08:00:00",
            "end_time": null,
            "rest_time": "00:00:00",
            "overnight_rest": false
        }))
        .await;

    res.assert_status(StatusCode::NOT_FOUND);

    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "WORKDAY_NOT_FOUND");
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_update_workday_with_wrong_body(ctx: &mut context::TestContext) {
    let res1 = ctx
        .authenticated_router
        .put("/workdays")
        .json(&json!({
            "date": "2026-01",
            "start_time": "08:00:00",
            "end_time": null,
            "rest_time": "00:00:00",
            "overnight_rest": false
        }))
        .await;

    res1.assert_status(StatusCode::BAD_REQUEST);

    let body1: ErrorBody = res1.json();
    assert_eq!(body1.error_code, "MISSING_ATTRIBUTE");

    let res2 = ctx
        .authenticated_router
        .put("/workdays")
        .json(&json!({
            "date": "2026-01-01",
            "start_time": "08:61",
            "end_time": null,
            "rest_time": "00:00:00",
            "overnight_rest": false
        }))
        .await;

    res2.assert_status(StatusCode::BAD_REQUEST);

    let body2: ErrorBody = res2.json();
    assert_eq!(body2.error_code, "MISSING_ATTRIBUTE");

    let res3 = ctx
        .authenticated_router
        .put("/workdays")
        .json(&json!({
            "date": null,
            "start_time": "08:00:00",
            "end_time": null,
            "rest_time": "00:00:00",
            "overnight_rest": false
        }))
        .await;

    res3.assert_status(StatusCode::BAD_REQUEST);

    let body3: ErrorBody = res3.json();
    assert_eq!(body3.error_code, "MISSING_ATTRIBUTE");

    let res4 = ctx
        .authenticated_router
        .put("/workdays")
        .json(&json!({
            "date": "2026-01-01",
            "start_time": null,
            "end_time": null,
            "rest_time": "00:00:00",
            "overnight_rest": false
        }))
        .await;

    res4.assert_status(StatusCode::BAD_REQUEST);

    let body4: ErrorBody = res4.json();
    assert_eq!(body4.error_code, "MISSING_ATTRIBUTE");

    let res5 = ctx
        .authenticated_router
        .put("/workdays")
        .json(&json!({
            "date": "2026-01-01",
            "start_time": "08:00:00",
            "end_time": null,
            "rest_time": null,
            "overnight_rest": false
        }))
        .await;

    res5.assert_status(StatusCode::BAD_REQUEST);

    let body5: ErrorBody = res5.json();
    assert_eq!(body5.error_code, "MISSING_ATTRIBUTE");
}
