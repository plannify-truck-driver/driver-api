use api::http::common::api_error::ErrorBody;
use axum::http::StatusCode;
use chrono::{Datelike, Months, NaiveDate, Utc};
use plannify_driver_api_core::domain::workday::{
    entities::{UpcomingWorkdayDocument, document_generation_cutoff},
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
        .get_pending_document_months(before, Some(ctx.authenticated_user_id))
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
