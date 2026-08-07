use api::http::common::api_error::ErrorBody;
use axum::http::StatusCode;
use plannify_driver_api_core::domain::driver_information::entities::{
    DriverInformation, DriverInformationType,
};
use serial_test::serial;
use test_context::test_context;

use crate::context;

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_driver_informations_unauthorized(ctx: &mut context::TestContext) {
    let res = ctx.unauthenticated_router.get("/informations").await;

    res.assert_status(StatusCode::UNAUTHORIZED);

    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "UNAUTHORIZED");
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_driver_informations_success(ctx: &mut context::TestContext) {
    let res = ctx.authenticated_router.get("/informations").await;

    res.assert_status(StatusCode::OK);

    let body: Vec<DriverInformation> = res.json();

    assert_eq!(
        body.len(),
        2,
        "only informations with show_at in the past and end_at in the future must be returned"
    );

    assert_eq!(body[0].information_type, DriverInformationType::INFO);
    assert_eq!(body[1].information_type, DriverInformationType::WARNING);
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_driver_informations_cache(ctx: &mut context::TestContext) {
    let res1 = ctx.authenticated_router.get("/informations").await;
    res1.assert_status(StatusCode::OK);
    let body1: Vec<DriverInformation> = res1.json();

    let res2 = ctx.authenticated_router.get("/informations").await;
    res2.assert_status(StatusCode::OK);
    let body2: Vec<DriverInformation> = res2.json();

    assert_eq!(body1.len(), 2);
    assert_eq!(body1.len(), body2.len());
    assert_eq!(body1[0].information_type, body2[0].information_type);
}
