use api::http::common::api_error::ErrorBody;
use axum::http::StatusCode;
use plannify_driver_api_core::domain::workday::port::WorkdayDatabaseRepository;
use serial_test::serial;
use test_context::test_context;

use crate::context;

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_delete_workday_unauthorized(ctx: &mut context::TestContext) {
    let res = ctx
        .unauthenticated_router
        .delete("/workdays/2027-01-02")
        .await;

    res.assert_status(StatusCode::UNAUTHORIZED);

    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "UNAUTHORIZED");
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_delete_workday_success(ctx: &mut context::TestContext) {
    let res = ctx
        .authenticated_router
        .delete("/workdays/2027-01-02")
        .await;

    res.assert_status(StatusCode::OK);

    ctx.repositories
        .workday_database_repository
        .delete_workday_garbage(
            ctx.authenticated_user_id,
            chrono::NaiveDate::from_ymd_opt(2027, 1, 2).unwrap(),
        )
        .await
        .unwrap();
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_delete_workday_not_found(ctx: &mut context::TestContext) {
    let res = ctx
        .authenticated_router
        .delete("/workdays/2028-01-01")
        .await;

    res.assert_status(StatusCode::NOT_FOUND);

    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "WORKDAY_NOT_FOUND");
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_delete_workday_duplicate(ctx: &mut context::TestContext) {
    let res = ctx
        .authenticated_router
        .delete("/workdays/2026-01-15")
        .await;

    res.assert_status(StatusCode::CONFLICT);

    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "WORKDAY_GARBAGE_ALREADY_EXISTS");
}