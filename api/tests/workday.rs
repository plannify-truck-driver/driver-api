use api::http::common::{api_error::ErrorBody, response::PaginatedResponse};
use axum::http::StatusCode;
use plannify_driver_api_core::domain::workday::{entities::Workday, port::WorkdayRepository};
use serde_json::json;
use test_context::test_context;

pub mod context;
pub mod helpers;

fn verify_workday_content(workday: Workday, expected_workday: Workday) {
    assert_eq!(workday.date, expected_workday.date);
    assert_eq!(workday.start_time, expected_workday.start_time);
    assert_eq!(workday.end_time, expected_workday.end_time);
    assert_eq!(workday.rest_time, expected_workday.rest_time);
    assert_eq!(workday.overnight_rest, expected_workday.overnight_rest);
}

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_get_all_workdays_month_unauthorized(ctx: &mut context::TestContext) {
    let res = ctx
        .unauthenticated_router
        .get("/driver/workdays/month?month=1&year=2026")
        .await;

    res.assert_status(StatusCode::UNAUTHORIZED);

    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "UNAUTHORIZED");
}

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_get_all_workdays_month_success(ctx: &mut context::TestContext) {
    let res = ctx
        .authenticated_router
        .get("/driver/workdays/month?month=1&year=2026")
        .await;

    res.assert_status(StatusCode::OK);

    let body: Vec<Workday> = res.json();
    assert_eq!(
        body.len(),
        2,
        "response array must contain exactly two workdays"
    );

    let expected_workday1 = Workday {
        date: chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        start_time: chrono::NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
        end_time: chrono::NaiveTime::from_hms_opt(17, 45, 0),
        rest_time: chrono::NaiveTime::from_hms_opt(1, 0, 0).unwrap(),
        overnight_rest: true,
    };
    let workday_json1 = &body[0];
    verify_workday_content(*workday_json1, expected_workday1);

    let expected_workday2 = Workday {
        date: chrono::NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
        start_time: chrono::NaiveTime::from_hms_opt(7, 40, 0).unwrap(),
        end_time: None,
        rest_time: chrono::NaiveTime::from_hms_opt(0, 30, 0).unwrap(),
        overnight_rest: false,
    };
    let workday_json2 = &body[1];
    verify_workday_content(*workday_json2, expected_workday2);
}

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_get_all_workdays_month_with_no_parameters(ctx: &mut context::TestContext) {
    let res1 = ctx.authenticated_router.get("/driver/workdays/month").await;

    res1.assert_status(StatusCode::BAD_REQUEST);

    let body1: ErrorBody = res1.json();
    assert_eq!(body1.error_code, "MISSING_ATTRIBUTE");

    let res2 = ctx
        .authenticated_router
        .get("/driver/workdays/month?month=1")
        .await;

    res2.assert_status(StatusCode::BAD_REQUEST);

    let body2: ErrorBody = res2.json();
    assert_eq!(body2.error_code, "MISSING_ATTRIBUTE");

    let res3 = ctx
        .authenticated_router
        .get("/driver/workdays/month?year=2026")
        .await;

    res3.assert_status(StatusCode::BAD_REQUEST);

    let body3: ErrorBody = res3.json();
    assert_eq!(body3.error_code, "MISSING_ATTRIBUTE");
}

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_get_all_workdays_month_with_wrong_parameters(ctx: &mut context::TestContext) {
    let res1 = ctx
        .authenticated_router
        .get("/driver/workdays/month?month=0&year=2026")
        .await;

    res1.assert_status(StatusCode::BAD_REQUEST);

    let body1: ErrorBody = res1.json();
    assert_eq!(body1.error_code, "QUERY_VALIDATION");

    let res2 = ctx
        .authenticated_router
        .get("/driver/workdays/month?month=13&year=2026")
        .await;

    res2.assert_status(StatusCode::BAD_REQUEST);

    let body2: ErrorBody = res2.json();
    assert_eq!(body2.error_code, "QUERY_VALIDATION");

    let res3 = ctx
        .authenticated_router
        .get("/driver/workdays/month?month=1&year=1800")
        .await;

    res3.assert_status(StatusCode::BAD_REQUEST);

    let body3: ErrorBody = res3.json();
    assert_eq!(body3.error_code, "QUERY_VALIDATION");

    let res4 = ctx
        .authenticated_router
        .get("/driver/workdays/month?month=1&year=2200")
        .await;

    res4.assert_status(StatusCode::BAD_REQUEST);

    let body4: ErrorBody = res4.json();
    assert_eq!(body4.error_code, "QUERY_VALIDATION");
}

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_get_all_workdays_period_unauthorized(ctx: &mut context::TestContext) {
    let res = ctx
        .unauthenticated_router
        .get("/driver/workdays?from=2026-01-01&to=2026-01-31&page=1&limit=10")
        .await;

    res.assert_status(StatusCode::UNAUTHORIZED);

    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "UNAUTHORIZED");
}

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_get_all_workdays_period_success(ctx: &mut context::TestContext) {
    let res = ctx
        .authenticated_router
        .get("/driver/workdays?from=2026-01-01&to=2026-12-31&page=1&limit=10")
        .await;

    res.assert_status(StatusCode::OK);

    let body: PaginatedResponse<Workday> = res.json();
    assert_eq!(body.page, 1, "page number should be 1");
    assert_eq!(body.total, 3, "total workdays should be 3");
    assert_eq!(
        body.data.len(),
        3,
        "response array must contain exactly three workdays"
    );

    let expected_workday1 = Workday {
        date: chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        start_time: chrono::NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
        end_time: chrono::NaiveTime::from_hms_opt(17, 45, 0),
        rest_time: chrono::NaiveTime::from_hms_opt(1, 0, 0).unwrap(),
        overnight_rest: true,
    };
    let workday_json1 = &body.data[0];
    verify_workday_content(*workday_json1, expected_workday1);

    let expected_workday2 = Workday {
        date: chrono::NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
        start_time: chrono::NaiveTime::from_hms_opt(7, 40, 0).unwrap(),
        end_time: None,
        rest_time: chrono::NaiveTime::from_hms_opt(0, 30, 0).unwrap(),
        overnight_rest: false,
    };
    let workday_json2 = &body.data[1];
    verify_workday_content(*workday_json2, expected_workday2);

    let expected_workday3 = Workday {
        date: chrono::NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
        start_time: chrono::NaiveTime::from_hms_opt(8, 40, 0).unwrap(),
        end_time: chrono::NaiveTime::from_hms_opt(17, 21, 0),
        rest_time: chrono::NaiveTime::from_hms_opt(1, 30, 0).unwrap(),
        overnight_rest: false,
    };
    let workday_json3 = &body.data[2];
    verify_workday_content(*workday_json3, expected_workday3);
}

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_get_all_workdays_period_with_no_parameters(ctx: &mut context::TestContext) {
    let res1 = ctx.authenticated_router.get("/driver/workdays").await;

    res1.assert_status(StatusCode::BAD_REQUEST);

    let body1: ErrorBody = res1.json();
    assert_eq!(body1.error_code, "MISSING_ATTRIBUTE");

    let res2 = ctx
        .authenticated_router
        .get("/driver/workdays?from=2026-01-01&to=2026-01-31")
        .await;

    res2.assert_status(StatusCode::BAD_REQUEST);

    let body2: ErrorBody = res2.json();
    assert_eq!(body2.error_code, "MISSING_ATTRIBUTE");

    let res3 = ctx
        .authenticated_router
        .get("/driver/workdays?page=1&limit=10")
        .await;

    res3.assert_status(StatusCode::BAD_REQUEST);

    let body3: ErrorBody = res3.json();
    assert_eq!(body3.error_code, "MISSING_ATTRIBUTE");
}

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_get_all_workdays_period_with_wrong_parameters(ctx: &mut context::TestContext) {
    let res1 = ctx
        .authenticated_router
        .get("/driver/workdays?from=2026-01-01&to=2026-01-31&page=0&limit=10")
        .await;

    res1.assert_status(StatusCode::BAD_REQUEST);

    let body1: ErrorBody = res1.json();
    assert_eq!(body1.error_code, "QUERY_VALIDATION");

    let res2 = ctx
        .authenticated_router
        .get("/driver/workdays?from=2026-01-01&to=2026-01-31&page=1&limit=0")
        .await;

    res2.assert_status(StatusCode::BAD_REQUEST);

    let body2: ErrorBody = res2.json();
    assert_eq!(body2.error_code, "QUERY_VALIDATION");

    let res3 = ctx
        .authenticated_router
        .get("/driver/workdays?from=2026-01-01&to=2026-01-31&page=1&limit=101")
        .await;

    res3.assert_status(StatusCode::BAD_REQUEST);

    let body3: ErrorBody = res3.json();
    assert_eq!(body3.error_code, "QUERY_VALIDATION");

    let res4 = ctx
        .authenticated_router
        .get("/driver/workdays?from=2026-01&to=2026-01-31&page=1&limit=10")
        .await;

    res4.assert_status(StatusCode::BAD_REQUEST);

    let body4: ErrorBody = res4.json();
    assert_eq!(body4.error_code, "MISSING_ATTRIBUTE");
}

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_create_workday_unauthorized(ctx: &mut context::TestContext) {
    let res = ctx
        .unauthenticated_router
        .post("/driver/workdays")
        .json(&json!({
            "date": "2026-01-01",
            "start_time": "08:00:00",
            "end_time": null,
            "rest_time": "00:00:00",
            "overnight_rest": false
        }))
        .await;

    res.assert_status(StatusCode::UNAUTHORIZED);

    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "UNAUTHORIZED");
}

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_create_workday_success(ctx: &mut context::TestContext) {
    let res = ctx
        .authenticated_router
        .post("/driver/workdays")
        .json(&json!({
            "date": "2027-03-01",
            "start_time": "08:00:00",
            "end_time": null,
            "rest_time": "00:00:00",
            "overnight_rest": false
        }))
        .await;

    res.assert_status(StatusCode::CREATED);

    let body: Workday = res.json();

    let expected_workday = Workday {
        date: chrono::NaiveDate::from_ymd_opt(2027, 3, 1).unwrap(),
        start_time: chrono::NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
        end_time: None,
        rest_time: chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
        overnight_rest: false,
    };
    verify_workday_content(body, expected_workday);

    ctx.repositories
        .workday_repository
        .delete_workday(
            ctx.authenticated_user_id,
            chrono::NaiveDate::from_ymd_opt(2027, 3, 1).unwrap(),
        )
        .await
        .unwrap();
}

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_create_workday_duplicate(ctx: &mut context::TestContext) {
    let res = ctx
        .authenticated_router
        .post("/driver/workdays")
        .json(&json!({
            "date": "2026-01-01",
            "start_time": "08:00:00",
            "end_time": null,
            "rest_time": "00:00:00",
            "overnight_rest": false
        }))
        .await;

    res.assert_status(StatusCode::CONFLICT);

    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "WORKDAY_ALREADY_EXISTS");
}

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_create_workday_wrong_body(ctx: &mut context::TestContext) {
    let res1 = ctx
        .authenticated_router
        .post("/driver/workdays")
        .json(&json!({
            "date": "2026-01",
            "start_time": "08:00:00",
            "end_time": null,
            "rest_time": "00:00:00",
            "overnight_rest": false
        }))
        .await;

    res1.assert_status(StatusCode::BAD_REQUEST);

    let body1: ErrorBody = res1.json();
    assert_eq!(body1.error_code, "MISSING_ATTRIBUTE");

    let res2 = ctx
        .authenticated_router
        .post("/driver/workdays")
        .json(&json!({
            "date": "2026-01-01",
            "start_time": "08:61",
            "end_time": null,
            "rest_time": "00:00:00",
            "overnight_rest": false
        }))
        .await;

    res2.assert_status(StatusCode::BAD_REQUEST);

    let body2: ErrorBody = res2.json();
    assert_eq!(body2.error_code, "MISSING_ATTRIBUTE");

    let res3 = ctx
        .authenticated_router
        .post("/driver/workdays")
        .json(&json!({
            "date": null,
            "start_time": "08:00:00",
            "end_time": null,
            "rest_time": "00:00:00",
            "overnight_rest": false
        }))
        .await;

    res3.assert_status(StatusCode::BAD_REQUEST);

    let body3: ErrorBody = res3.json();
    assert_eq!(body3.error_code, "MISSING_ATTRIBUTE");

    let res4 = ctx
        .authenticated_router
        .post("/driver/workdays")
        .json(&json!({
            "date": "2026-01-01",
            "start_time": null,
            "end_time": null,
            "rest_time": "00:00:00",
            "overnight_rest": false
        }))
        .await;

    res4.assert_status(StatusCode::BAD_REQUEST);

    let body4: ErrorBody = res4.json();
    assert_eq!(body4.error_code, "MISSING_ATTRIBUTE");

    let res5 = ctx
        .authenticated_router
        .post("/driver/workdays")
        .json(&json!({
            "date": "2026-01-01",
            "start_time": "08:00:00",
            "end_time": null,
            "rest_time": null,
            "overnight_rest": false
        }))
        .await;

    res5.assert_status(StatusCode::BAD_REQUEST);

    let body5: ErrorBody = res5.json();
    assert_eq!(body5.error_code, "MISSING_ATTRIBUTE");
}

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_create_workday_duplicate_garbage(ctx: &mut context::TestContext) {
    let res = ctx
        .authenticated_router
        .post("/driver/workdays")
        .json(&json!({
            "date": "2026-01-15",
            "start_time": "08:00:00",
            "end_time": null,
            "rest_time": "00:00:00",
            "overnight_rest": false
        }))
        .await;

    res.assert_status(StatusCode::CONFLICT);

    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "WORKDAY_ALREADY_EXISTS");
}

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_update_workday_unauthorized(ctx: &mut context::TestContext) {
    let res = ctx
        .unauthenticated_router
        .put("/driver/workdays")
        .json(&json!({
            "date": "2026-01-01",
            "start_time": "08:00:00",
            "end_time": null,
            "rest_time": "00:00:00",
            "overnight_rest": false
        }))
        .await;

    res.assert_status(StatusCode::UNAUTHORIZED);

    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "UNAUTHORIZED");
}

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_update_workday_success(ctx: &mut context::TestContext) {
    let res = ctx
        .authenticated_router
        .put("/driver/workdays")
        .json(&json!({
            "date": "2027-01-01",
            "start_time": "08:00:00",
            "end_time": null,
            "rest_time": "00:00:00",
            "overnight_rest": false
        }))
        .await;

    res.assert_status(StatusCode::OK);

    let body: Workday = res.json();

    let expected_workday = Workday {
        date: chrono::NaiveDate::from_ymd_opt(2027, 1, 1).unwrap(),
        start_time: chrono::NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
        end_time: None,
        rest_time: chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
        overnight_rest: false,
    };
    verify_workday_content(body, expected_workday);
}

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_update_workday_not_found(ctx: &mut context::TestContext) {
    let res = ctx
        .authenticated_router
        .put("/driver/workdays")
        .json(&json!({
            "date": "2028-01-01",
            "start_time": "08:00:00",
            "end_time": null,
            "rest_time": "00:00:00",
            "overnight_rest": false
        }))
        .await;

    res.assert_status(StatusCode::NOT_FOUND);

    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "WORKDAY_NOT_FOUND");
}

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_update_workday_wrong_body(ctx: &mut context::TestContext) {
    let res1 = ctx
        .authenticated_router
        .put("/driver/workdays")
        .json(&json!({
            "date": "2026-01",
            "start_time": "08:00:00",
            "end_time": null,
            "rest_time": "00:00:00",
            "overnight_rest": false
        }))
        .await;

    res1.assert_status(StatusCode::BAD_REQUEST);

    let body1: ErrorBody = res1.json();
    assert_eq!(body1.error_code, "MISSING_ATTRIBUTE");

    let res2 = ctx
        .authenticated_router
        .put("/driver/workdays")
        .json(&json!({
            "date": "2026-01-01",
            "start_time": "08:61",
            "end_time": null,
            "rest_time": "00:00:00",
            "overnight_rest": false
        }))
        .await;

    res2.assert_status(StatusCode::BAD_REQUEST);

    let body2: ErrorBody = res2.json();
    assert_eq!(body2.error_code, "MISSING_ATTRIBUTE");

    let res3 = ctx
        .authenticated_router
        .put("/driver/workdays")
        .json(&json!({
            "date": null,
            "start_time": "08:00:00",
            "end_time": null,
            "rest_time": "00:00:00",
            "overnight_rest": false
        }))
        .await;

    res3.assert_status(StatusCode::BAD_REQUEST);

    let body3: ErrorBody = res3.json();
    assert_eq!(body3.error_code, "MISSING_ATTRIBUTE");

    let res4 = ctx
        .authenticated_router
        .put("/driver/workdays")
        .json(&json!({
            "date": "2026-01-01",
            "start_time": null,
            "end_time": null,
            "rest_time": "00:00:00",
            "overnight_rest": false
        }))
        .await;

    res4.assert_status(StatusCode::BAD_REQUEST);

    let body4: ErrorBody = res4.json();
    assert_eq!(body4.error_code, "MISSING_ATTRIBUTE");

    let res5 = ctx
        .authenticated_router
        .put("/driver/workdays")
        .json(&json!({
            "date": "2026-01-01",
            "start_time": "08:00:00",
            "end_time": null,
            "rest_time": null,
            "overnight_rest": false
        }))
        .await;

    res5.assert_status(StatusCode::BAD_REQUEST);

    let body5: ErrorBody = res5.json();
    assert_eq!(body5.error_code, "MISSING_ATTRIBUTE");
}

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_delete_workday_unauthorized(ctx: &mut context::TestContext) {
    let res = ctx
        .unauthenticated_router
        .delete("/driver/workdays/2027-01-02")
        .await;

    res.assert_status(StatusCode::UNAUTHORIZED);

    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "UNAUTHORIZED");
}

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_delete_workday_success(ctx: &mut context::TestContext) {
    let res = ctx
        .authenticated_router
        .delete("/driver/workdays/2027-01-02")
        .await;

    res.assert_status(StatusCode::OK);

    ctx.repositories
        .workday_repository
        .delete_workday_garbage(
            ctx.authenticated_user_id,
            chrono::NaiveDate::from_ymd_opt(2027, 1, 2).unwrap(),
        )
        .await
        .unwrap();
}

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_delete_workday_not_found(ctx: &mut context::TestContext) {
    let res = ctx
        .authenticated_router
        .delete("/driver/workdays/2028-01-01")
        .await;

    res.assert_status(StatusCode::NOT_FOUND);

    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "WORKDAY_NOT_FOUND");
}

#[test_context(context::TestContext)]
#[tokio::test]
async fn test_delete_workday_duplicate(ctx: &mut context::TestContext) {
    let res = ctx
        .authenticated_router
        .delete("/driver/workdays/2026-01-15")
        .await;

    res.assert_status(StatusCode::CONFLICT);

    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "WORKDAY_GARBAGE_ALREADY_EXISTS");
}
