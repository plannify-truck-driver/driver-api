use api::http::common::api_error::ErrorBody;
use axum::http::StatusCode;
use serial_test::serial;
use test_context::test_context;
use serde_json::json;

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

    ctx.authenticated_router.post("/workdays").json(&json!({
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
}