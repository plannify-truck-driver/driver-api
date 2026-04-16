use api::http::common::api_error::ErrorBody;
use axum::http::StatusCode;
use plannify_driver_api_core::domain::workday::{
    entities::WorkdayGarbage, port::WorkdayDatabaseRepository,
};
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

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_delete_workday_cross_user_isolation(ctx: &mut context::TestContext) {
    // 2027-01-01 belongs to User A. User B must receive 404 when trying to soft-delete it.
    let other_router = ctx.create_authenticated_router_with_different_user().await;

    let res = other_router.delete("/workdays/2027-01-01").await;

    res.assert_status(StatusCode::NOT_FOUND);
    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "WORKDAY_NOT_FOUND");
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_delete_workday_disappears_from_get(ctx: &mut context::TestContext) {
    // 2026-02-01 is accessible before deletion
    ctx.authenticated_router
        .get("/workdays/2026-02-01")
        .await
        .assert_status(StatusCode::OK);

    // Soft-delete it
    ctx.authenticated_router
        .delete("/workdays/2026-02-01")
        .await
        .assert_status(StatusCode::OK);

    // It must no longer be reachable
    let res = ctx.authenticated_router.get("/workdays/2026-02-01").await;
    res.assert_status(StatusCode::NOT_FOUND);
    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "WORKDAY_NOT_FOUND");

    // Cleanup: remove the garbage marker to restore visibility for other tests
    ctx.repositories
        .workday_database_repository
        .delete_workday_garbage(
            ctx.authenticated_user_id,
            chrono::NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
        )
        .await
        .ok();
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_delete_workday_appears_in_garbage(ctx: &mut context::TestContext) {
    // Soft-delete 2026-01-31
    ctx.authenticated_router
        .delete("/workdays/2026-01-31")
        .await
        .assert_status(StatusCode::OK);

    // It must now appear in the garbage list
    let res = ctx.authenticated_router.get("/workdays/garbage").await;
    res.assert_status(StatusCode::OK);
    let body: Vec<WorkdayGarbage> = res.json();
    assert!(
        body.iter()
            .any(|g| g.workday_date == chrono::NaiveDate::from_ymd_opt(2026, 1, 31).unwrap()),
        "2026-01-31 must be present in the garbage list after soft-deletion"
    );

    // Cleanup: remove garbage marker so other tests still see this workday
    ctx.repositories
        .workday_database_repository
        .delete_workday_garbage(
            ctx.authenticated_user_id,
            chrono::NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
        )
        .await
        .ok();
}
