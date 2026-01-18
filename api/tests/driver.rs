use api::http::common::api_error::ErrorBody;
use plannify_driver_api_core::domain::driver::{
    entities::DriverRestPeriod, port::DriverRepository,
};
use reqwest::StatusCode;
use serde_json::json;
use serial_test::serial;
use test_context::test_context;

pub mod context;
pub mod helpers;

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_all_rest_periods_unauthorized(ctx: &mut context::TestContext) {
    let res = ctx.unauthenticated_router.get("/driver/rest-periods").await;

    res.assert_status(StatusCode::UNAUTHORIZED);

    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "UNAUTHORIZED");
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_all_rest_periods_success(ctx: &mut context::TestContext) {
    let res = ctx.authenticated_router.get("/driver/rest-periods").await;

    res.assert_status(StatusCode::OK);

    let body: Vec<DriverRestPeriod> = res.json();
    assert_eq!(body.len(), 2);
    assert_eq!(body[0].start, "00:00:00".parse().unwrap());
    assert_eq!(body[0].end, "00:59:59".parse().unwrap());
    assert_eq!(body[0].rest, "01:00:00".parse().unwrap());
    assert_eq!(body[1].start, "01:00:00".parse().unwrap());
    assert_eq!(body[1].end, "23:59:59".parse().unwrap());
    assert_eq!(body[1].rest, "01:00:00".parse().unwrap());
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_set_rest_periods_unauthorized(ctx: &mut context::TestContext) {
    let res = ctx
        .unauthenticated_router
        .post("/driver/rest-periods")
        .json(&json!({
            "rest_periods": [
                {
                    "start": "00:00:00",
                    "end": "00:59:59",
                    "rest": "01:00:00"
                },
                {
                    "start": "01:00:00",
                    "end": "23:59:59",
                    "rest": "01:00:00"
                }
            ]
        }))
        .await;

    res.assert_status(StatusCode::UNAUTHORIZED);

    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "UNAUTHORIZED");
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_set_rest_periods_success(ctx: &mut context::TestContext) {
    let res = ctx
        .authenticated_router
        .post("/driver/rest-periods")
        .json(&json!({
            "rest_periods": [
                {
                    "start": "00:00:00",
                    "end": "00:59:59",
                    "rest": "01:00:00"
                },
                {
                    "start": "01:00:00",
                    "end": "23:59:59",
                    "rest": "01:00:00"
                }
            ]
        }))
        .await;

    res.assert_status(StatusCode::CREATED);
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_set_rest_periods_with_wrong_body_first_start(ctx: &mut context::TestContext) {
    let res = ctx
        .authenticated_router
        .post("/driver/rest-periods")
        .json(&json!({
            "rest_periods": [
                {
                    "start": "00:10:00",
                    "end": "00:59:59",
                    "rest": "01:00:00"
                },
                {
                    "start": "01:00:00",
                    "end": "23:59:59",
                    "rest": "01:00:00"
                }
            ]
        }))
        .await;

    res.assert_status(StatusCode::BAD_REQUEST);
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_set_rest_periods_with_wrong_body_last_end(ctx: &mut context::TestContext) {
    let res = ctx
        .authenticated_router
        .post("/driver/rest-periods")
        .json(&json!({
            "rest_periods": [
                {
                    "start": "01:00:00",
                    "end": "23:50:00",
                    "rest": "01:00:00"
                },
                {
                    "start": "00:00:00",
                    "end": "00:59:59",
                    "rest": "01:00:00"
                }
            ]
        }))
        .await;

    res.assert_status(StatusCode::BAD_REQUEST);
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_set_rest_periods_with_wrong_body_big_gap(ctx: &mut context::TestContext) {
    let res = ctx
        .authenticated_router
        .post("/driver/rest-periods")
        .json(&json!({
            "rest_periods": [
                {
                    "start": "00:00:00",
                    "end": "01:00:00",
                    "rest": "01:00:00"
                },
                {
                    "start": "01:00:00",
                    "end": "23:50:00",
                    "rest": "01:00:00"
                }
            ]
        }))
        .await;

    res.assert_status(StatusCode::BAD_REQUEST);
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_delete_rest_periods_unauthorized(ctx: &mut context::TestContext) {
    let res = ctx
        .unauthenticated_router
        .delete("/driver/rest-periods")
        .await;

    res.assert_status(StatusCode::UNAUTHORIZED);

    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "UNAUTHORIZED");
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_delete_rest_periods_success(ctx: &mut context::TestContext) {
    let res = ctx
        .authenticated_router
        .delete("/driver/rest-periods")
        .await;

    res.assert_status(StatusCode::OK);

    ctx.repositories
        .driver_repository
        .set_driver_rest_periods(
            ctx.authenticated_user_id,
            vec![
                DriverRestPeriod {
                    start: "00:00:00".parse().unwrap(),
                    end: "00:59:59".parse().unwrap(),
                    rest: "01:00:00".parse().unwrap(),
                },
                DriverRestPeriod {
                    start: "01:00:00".parse().unwrap(),
                    end: "23:59:59".parse().unwrap(),
                    rest: "01:00:00".parse().unwrap(),
                },
            ],
        )
        .await
        .unwrap();
}
