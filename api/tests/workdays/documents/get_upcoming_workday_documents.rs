use api::http::common::api_error::ErrorBody;
use axum::http::StatusCode;
use chrono::{Datelike, Months, NaiveDate, Utc};
use plannify_driver_api_core::domain::workday::{
    entities::{
        UpcomingWorkdayDocument, document_generation_account_cutoff, document_generation_cutoff,
    },
    port::WorkdayDatabaseRepository,
};
use serial_test::serial;
use test_context::test_context;

use crate::context;

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_upcoming_workday_documents_unauthorized(ctx: &mut context::TestContext) {
    let res = ctx
        .unauthenticated_router
        .get("/workdays/documents/upcoming")
        .await;

    res.assert_status(StatusCode::UNAUTHORIZED);

    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "UNAUTHORIZED");
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_upcoming_workday_documents_success(ctx: &mut context::TestContext) {
    let today = Utc::now().date_naive();
    let next_run =
        NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap() + Months::new(1);
    let before = document_generation_cutoff(next_run, 3);

    let expected: Vec<UpcomingWorkdayDocument> = ctx
        .repositories
        .workday_database_repository
        .get_pending_document_months(
            before,
            document_generation_account_cutoff(next_run),
            Some(ctx.authenticated_user_id),
        )
        .await
        .unwrap()
        .into_iter()
        .map(|(_, month, year)| UpcomingWorkdayDocument { month, year })
        .collect();

    let res = ctx
        .authenticated_router
        .get("/workdays/documents/upcoming")
        .await;

    res.assert_status(StatusCode::OK);

    let body: Vec<UpcomingWorkdayDocument> = res.json();
    assert_eq!(body, expected);
    assert!(
        body.iter()
            .all(|d| NaiveDate::from_ymd_opt(d.year, d.month as u32, 1).unwrap() < before),
        "only months before the next job cutoff should be returned"
    );
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_upcoming_workday_documents_account_too_recent(ctx: &mut context::TestContext) {
    let res = ctx
        .authenticated_router
        .get("/workdays/documents/upcoming")
        .await;
    res.assert_status(StatusCode::OK);
    let body: Vec<UpcomingWorkdayDocument> = res.json();
    assert!(
        !body.is_empty(),
        "test user must have pending months, otherwise this test proves nothing"
    );

    let original_created_at: chrono::DateTime<Utc> =
        sqlx::query_scalar("SELECT created_at FROM drivers WHERE pk_driver_id = $1")
            .bind(ctx.authenticated_user_id)
            .fetch_one(&ctx.repositories.pool)
            .await
            .unwrap();

    // Account created less than 6 months before the next job run
    sqlx::query(
        "UPDATE drivers SET created_at = NOW() - INTERVAL '4 months' WHERE pk_driver_id = $1",
    )
    .bind(ctx.authenticated_user_id)
    .execute(&ctx.repositories.pool)
    .await
    .unwrap();

    // Upcoming documents are cached per driver: clear them to see the new account age
    let mut redis_conn = ctx.repositories.redis_manager.clone();
    let _: () = redis::cmd("FLUSHDB")
        .query_async(&mut redis_conn)
        .await
        .unwrap();

    let res = ctx
        .authenticated_router
        .get("/workdays/documents/upcoming")
        .await;

    sqlx::query("UPDATE drivers SET created_at = $2 WHERE pk_driver_id = $1")
        .bind(ctx.authenticated_user_id)
        .bind(original_created_at)
        .execute(&ctx.repositories.pool)
        .await
        .unwrap();

    res.assert_status(StatusCode::OK);
    let body: Vec<UpcomingWorkdayDocument> = res.json();
    assert!(
        body.is_empty(),
        "no document should be generated for an account younger than 6 months"
    );
}
