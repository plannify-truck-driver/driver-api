use api::http::common::api_error::ErrorBody;
use axum::http::StatusCode;
use plannify_driver_api_core::domain::workday::entities::WorkdayDocumentInformation;
use serial_test::serial;
use test_context::test_context;

use crate::{context, workdays::verify_workday_document_content};

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
async fn test_get_workday_documents_by_year_in_database_success(ctx: &mut context::TestContext) {
    let res = ctx
        .authenticated_router
        .get("/workdays/documents/2026")
        .await;

    res.assert_status(StatusCode::OK);

    let body: Vec<WorkdayDocumentInformation> = res.json();

    assert_eq!(
        body.len(),
        2,
        "there should be exactly two months available"
    );

    let expected_workday_document_1 = WorkdayDocumentInformation {
        month: 1,
        year: 2026,
        generated_at: None,
    };
    let workday_document_1 = body[0];
    verify_workday_document_content(workday_document_1, expected_workday_document_1);

    let expected_workday_document_2 = WorkdayDocumentInformation {
        month: 2,
        year: 2026,
        generated_at: Some("2026-03-01T10:00:00Z".parse().unwrap()),
    };
    let workday_document_2 = body[1];
    verify_workday_document_content(workday_document_2, expected_workday_document_2);
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_workday_documents_by_year_in_s3_success(ctx: &mut context::TestContext) {
    let res = ctx
        .authenticated_router
        .get("/workdays/documents/2031")
        .await;

    res.assert_status(StatusCode::OK);

    let body: Vec<WorkdayDocumentInformation> = res.json();

    assert_eq!(body.len(), 1, "there should be exactly one month available");

    let expected_workday_document = WorkdayDocumentInformation {
        month: 1,
        year: 2031,
        generated_at: Some("2031-02-01T10:00:00Z".parse().unwrap()),
    };
    let workday_document = body[0];
    verify_workday_document_content(workday_document, expected_workday_document);
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

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_workday_documents_by_year_cross_user_isolation(ctx: &mut context::TestContext) {
    // User B has no workdays or documents in 2031 — they must see an empty list,
    // not User A's seeded document (2031/01).
    let other_router = ctx.create_authenticated_router_with_different_user().await;

    let res = other_router.get("/workdays/documents/2031").await;

    res.assert_status(StatusCode::OK);
    let body: Vec<WorkdayDocumentInformation> = res.json();
    assert!(
        body.is_empty(),
        "User B must see no documents for 2031, got {}",
        body.len()
    );
}
