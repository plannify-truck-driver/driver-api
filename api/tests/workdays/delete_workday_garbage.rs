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
async fn test_delete_workday_garbage_unauthorized(ctx: &mut context::TestContext) {
    let res = ctx
        .unauthenticated_router
        .delete("/workdays/garbage/2026-01-15")
        .await;

    res.assert_status(StatusCode::UNAUTHORIZED);

    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "UNAUTHORIZED");
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_delete_workday_garbage_success(ctx: &mut context::TestContext) {
    let res = ctx
        .authenticated_router
        .delete("/workdays/garbage/2026-01-15")
        .await;

    res.assert_status(StatusCode::OK);

    ctx.repositories
        .workday_database_repository
        .create_workday_garbage(
            ctx.authenticated_user_id,
            chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2026, 3, 11).unwrap(),
            Some(
                chrono::NaiveDateTime::new(
                    chrono::NaiveDate::from_ymd_opt(2026, 2, 10).unwrap(),
                    chrono::NaiveTime::from_hms_opt(11, 30, 0).unwrap(),
                )
                .and_utc(),
            ),
        )
        .await
        .unwrap();
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_delete_workday_garbage_not_found(ctx: &mut context::TestContext) {
    let res = ctx
        .authenticated_router
        .delete("/workdays/garbage/2028-01-01")
        .await;

    res.assert_status(StatusCode::NOT_FOUND);

    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "WORKDAY_GARBAGE_NOT_FOUND");
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_delete_workday_garbage_cross_user_isolation(ctx: &mut context::TestContext) {
    // 2024-01-01 is in User A's garbage. User B must receive 404 when trying to restore it.
    let other_router = ctx.create_authenticated_router_with_different_user().await;

    let res = other_router.delete("/workdays/garbage/2024-01-01").await;

    res.assert_status(StatusCode::NOT_FOUND);
    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "WORKDAY_GARBAGE_NOT_FOUND");
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_restore_workday_reappears_in_get(ctx: &mut context::TestContext) {
    // 2026-01-15 is currently soft-deleted — it must not be accessible
    ctx.authenticated_router
        .get("/workdays/2026-01-15")
        .await
        .assert_status(StatusCode::NOT_FOUND);

    // Restore it by removing the garbage marker
    ctx.authenticated_router
        .delete("/workdays/garbage/2026-01-15")
        .await
        .assert_status(StatusCode::OK);

    // It must now be accessible again
    ctx.authenticated_router
        .get("/workdays/2026-01-15")
        .await
        .assert_status(StatusCode::OK);

    // And it must no longer appear in the garbage list
    let garbage_res = ctx.authenticated_router.get("/workdays/garbage").await;
    garbage_res.assert_status(StatusCode::OK);
    let garbage: Vec<WorkdayGarbage> = garbage_res.json();
    assert!(
        !garbage
            .iter()
            .any(|g| g.workday_date == chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap()),
        "2026-01-15 must no longer appear in the garbage list after restoration"
    );

    // Cleanup: put the garbage entry back so other tests that rely on it still pass
    ctx.repositories
        .workday_database_repository
        .create_workday_garbage(
            ctx.authenticated_user_id,
            chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2026, 3, 11).unwrap(),
            Some(
                chrono::NaiveDateTime::new(
                    chrono::NaiveDate::from_ymd_opt(2026, 2, 10).unwrap(),
                    chrono::NaiveTime::from_hms_opt(11, 30, 0).unwrap(),
                )
                .and_utc(),
            ),
        )
        .await
        .ok();
}
