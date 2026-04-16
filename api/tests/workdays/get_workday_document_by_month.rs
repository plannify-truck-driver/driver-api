use api::http::common::api_error::ErrorBody;
use axum::http::StatusCode;
use serial_test::serial;
use test_context::test_context;

use crate::context;

// --- Authorization ---

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_workday_document_by_month_unauthorized(ctx: &mut context::TestContext) {
    let res = ctx
        .unauthenticated_router
        .get("/workdays/documents/2026/2")
        .await;

    res.assert_status(StatusCode::UNAUTHORIZED);

    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "UNAUTHORIZED");
}

// --- Input validation ---

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_workday_document_by_month_invalid_month_zero(ctx: &mut context::TestContext) {
    let res = ctx
        .authenticated_router
        .get("/workdays/documents/2026/0")
        .await;

    res.assert_status(StatusCode::BAD_REQUEST);

    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "INVALID_MONTH");
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_workday_document_by_month_invalid_month_thirteen(ctx: &mut context::TestContext) {
    let res = ctx
        .authenticated_router
        .get("/workdays/documents/2026/13")
        .await;

    res.assert_status(StatusCode::BAD_REQUEST);

    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "INVALID_MONTH");
}

// --- S3 path: workday_documents record exists (seeded via SQL + MinIO) ---

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_workday_document_by_month_returns_pdf_when_record_exists(
    ctx: &mut context::TestContext,
) {
    // (2026, 02): workday_documents row in test-dataset.sql + file uploaded to MinIO in setup.
    let res = ctx
        .authenticated_router
        .get("/workdays/documents/2026/2")
        .await;

    res.assert_status(StatusCode::OK);

    let content_type = res
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    assert!(
        content_type.contains("application/pdf"),
        "expected application/pdf, got: {}",
        content_type
    );
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_workday_document_by_month_returns_pdf_for_second_seeded_month(
    ctx: &mut context::TestContext,
) {
    // (2027, 01): second workday_documents row in test-dataset.sql.
    let res = ctx
        .authenticated_router
        .get("/workdays/documents/2027/1")
        .await;

    res.assert_status(StatusCode::OK);

    let content_type = res
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    assert!(
        content_type.contains("application/pdf"),
        "expected application/pdf, got: {}",
        content_type
    );
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_workday_document_by_month_not_accessible_by_other_user(
    ctx: &mut context::TestContext,
) {
    // Fetch baseline document for the seeded user (S3 path).
    let owner_res = ctx
        .authenticated_router
        .get("/workdays/documents/2026/2")
        .await;
    owner_res.assert_status(StatusCode::OK);
    let owner_pdf = owner_res.as_bytes().to_vec();

    // The second user (123e4567-e89b-12d3-a456-426614174001) has no workday_documents record.
    // The endpoint may still return 200 via gRPC generation, but it must not return
    // the first user's seeded S3 document bytes.
    let other_router = ctx.create_authenticated_router_with_different_user().await;

    let res = other_router.get("/workdays/documents/2026/2").await;

    match res.status_code() {
        StatusCode::OK => {
            let content_type = res
                .headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");

            assert!(
                content_type.contains("application/pdf"),
                "expected application/pdf on grpc generation, got: {}",
                content_type
            );

            assert_ne!(
                res.as_bytes().as_ref(),
                owner_pdf.as_slice(),
                "another user's request must not return the seeded S3 PDF bytes"
            );
        }
        status => panic!("unexpected status for other user: {}", status),
    }
}
