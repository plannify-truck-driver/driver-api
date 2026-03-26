use api::http::common::api_error::ErrorBody;
use axum::http::StatusCode;
use plannify_driver_api_core::domain::workday::entities::WorkdayDocument;
use serial_test::serial;
use test_context::test_context;

use crate::{context, workdays::verify_workday_document_content};

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_workday_documents_unauthorized(ctx: &mut context::TestContext) {
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
async fn test_get_workday_documents_success(ctx: &mut context::TestContext) {
    let res = ctx
        .authenticated_router
        .get("/workdays/documents/year")
        .await;

    res.assert_status(StatusCode::OK);

    let body: Vec<i32> = res.json();

    assert_eq!(
        body.len(),
        3,
        "there should be exactly three years available"
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
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_workday_documents_by_year_unauthorized(ctx: &mut context::TestContext) {
    let res = ctx
        .unauthenticated_router
        .get("/workdays/documents/2026")
        .await;

    res.assert_status(StatusCode::UNAUTHORIZED);

    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "UNAUTHORIZED");
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_workday_documents_by_year_success(ctx: &mut context::TestContext) {
    let res = ctx
        .authenticated_router
        .get("/workdays/documents/2026")
        .await;

    res.assert_status(StatusCode::OK);

    let body: Vec<WorkdayDocument> = res.json();

    assert_eq!(
        body.len(),
        2,
        "there should be exactly two months available"
    );

    let expected_workday_document_1 = WorkdayDocument {
        month: 1,
        year: 2026,
        generated_at: None,
    };
    let workday_document_1 = body[0];
    verify_workday_document_content(workday_document_1, expected_workday_document_1);

    let expected_workday_document_2 = WorkdayDocument {
        month: 2,
        year: 2026,
        generated_at: None,
    };
    let workday_document_2 = body[1];
    verify_workday_document_content(workday_document_2, expected_workday_document_2);
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_workday_documents_by_year_without_garbage(ctx: &mut context::TestContext) {
    let res = ctx
        .authenticated_router
        .get("/workdays/documents/2024")
        .await;

    res.assert_status(StatusCode::OK);

    let body: Vec<i32> = res.json();

    assert_eq!(
        body.len(),
        0,
        "there should be exactly zero months available"
    );
}
