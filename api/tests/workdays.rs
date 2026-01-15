use api::http::common::api_error::ErrorBody;
use axum::http::StatusCode;
use serde_json::Value;
use test_context::test_context;

pub mod context;
pub mod helpers;

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_get_all_workdays_month_unauthorized(ctx: &mut context::TestContext) {
    let res = ctx
        .unauthenticated_router
        .get("/driver/workdays/month?month=1&year=2026")
        .await;

    res.assert_status(StatusCode::UNAUTHORIZED);

    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "UNAUTHORIZED");
}

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_get_all_workdays_month_success(ctx: &mut context::TestContext) {
    let res = ctx
        .authenticated_router
        .get("/driver/workdays/month?month=1&year=2026")
        .await;

    res.assert_status(StatusCode::OK);

    let body: Value = res.json();
    assert!(body.is_array(), "response must be an array");
    assert!(
        body.as_array().unwrap().len() == 3,
        "response array must contain exactly three workdays"
    );
    assert_eq!(
        body[0].get("date").and_then(|v| v.as_str()),
        Some("2026-01-01")
    );
    assert_eq!(
        body[0].get("start_time").and_then(|v| v.as_str()),
        Some("08:00:00")
    );
    assert_eq!(
        body[0].get("end_time").and_then(|v| v.as_str()),
        Some("17:45:00")
    );
    assert_eq!(
        body[0].get("rest_time").and_then(|v| v.as_str()),
        Some("01:00:00")
    );
    assert_eq!(
        body[0].get("overnight_rest").and_then(|v| v.as_bool()),
        Some(false)
    );
}
