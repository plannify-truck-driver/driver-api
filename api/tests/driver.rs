use api::http::common::api_error::ErrorBody;
use api::http::common::middleware::auth::entities::AccessClaims;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use plannify_driver_api_core::domain::driver::{
    entities::{CreateDriverResponse, DriverRestPeriod},
    port::DriverDatabaseRepository,
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
    let res = ctx.unauthenticated_router.get("/rest-periods").await;

    res.assert_status(StatusCode::UNAUTHORIZED);

    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "UNAUTHORIZED");
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_all_rest_periods_success(ctx: &mut context::TestContext) {
    let res = ctx.authenticated_router.get("/rest-periods").await;

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
        .post("/rest-periods")
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
        .post("/rest-periods")
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
        .post("/rest-periods")
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
        .post("/rest-periods")
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
        .post("/rest-periods")
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
    let res = ctx.unauthenticated_router.delete("/rest-periods").await;

    res.assert_status(StatusCode::UNAUTHORIZED);

    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "UNAUTHORIZED");
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_delete_rest_periods_success(ctx: &mut context::TestContext) {
    let res = ctx.authenticated_router.delete("/rest-periods").await;

    res.assert_status(StatusCode::OK);

    ctx.repositories
        .driver_database_repository
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

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_get_all_rest_periods_cross_user_isolation(ctx: &mut context::TestContext) {
    // User A has 2 rest periods set in the test dataset.
    // User B has rest_json = NULL, so they must see a different (empty) result.
    let other_router = ctx.create_authenticated_router_with_different_user().await;

    let user_a_res = ctx.authenticated_router.get("/rest-periods").await;
    user_a_res.assert_status(StatusCode::OK);
    let user_a_body: Vec<DriverRestPeriod> = user_a_res.json();
    assert_eq!(user_a_body.len(), 2, "User A should have 2 rest periods");

    let user_b_res = other_router.get("/rest-periods").await;
    user_b_res.assert_status(StatusCode::OK);
    let user_b_body: Vec<DriverRestPeriod> = user_b_res.json();
    assert_ne!(
        user_b_body.len(),
        user_a_body.len(),
        "User B must not see User A's rest periods"
    );
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_delete_rest_periods_then_get_empty(ctx: &mut context::TestContext) {
    // After deleting rest periods, GET must return an empty list
    ctx.authenticated_router
        .delete("/rest-periods")
        .await
        .assert_status(StatusCode::OK);

    let res = ctx.authenticated_router.get("/rest-periods").await;
    res.assert_status(StatusCode::OK);
    let body: Vec<DriverRestPeriod> = res.json();
    assert!(
        body.is_empty(),
        "rest periods must be empty after deletion, got {}",
        body.len()
    );

    // Cleanup: restore the original periods
    ctx.repositories
        .driver_database_repository
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

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_set_rest_periods_empty_array(ctx: &mut context::TestContext) {
    // An empty rest_periods array must be rejected (no period starts at 00:00:00)
    let res = ctx
        .authenticated_router
        .post("/rest-periods")
        .json(&json!({ "rest_periods": [] }))
        .await;

    res.assert_status(StatusCode::BAD_REQUEST);
}

fn decode_access_token(token: &str) -> AccessClaims {
    decode::<AccessClaims>(
        token,
        &DecodingKey::from_secret("test-secret-key".as_ref()),
        &Validation::new(Algorithm::HS256),
    )
    .expect("Failed to decode access token")
    .claims
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_update_driver_info_unauthorized(ctx: &mut context::TestContext) {
    let res = ctx
        .unauthenticated_router
        .patch("/me")
        .json(&json!({}))
        .await;

    res.assert_status(StatusCode::UNAUTHORIZED);
    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "UNAUTHORIZED");
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_update_driver_info_empty_body_no_change(ctx: &mut context::TestContext) {
    let original = ctx
        .repositories
        .driver_database_repository
        .get_driver_by_id(ctx.authenticated_user_id)
        .await
        .unwrap()
        .unwrap();

    let res = ctx.authenticated_router.patch("/me").json(&json!({})).await;

    res.assert_status(StatusCode::OK);
    let body: CreateDriverResponse = res.json();
    assert!(!body.access_token.is_empty());

    let driver = ctx
        .repositories
        .driver_database_repository
        .get_driver_by_id(ctx.authenticated_user_id)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(driver.firstname, original.firstname);
    assert_eq!(driver.lastname, original.lastname);
    assert_eq!(driver.email, original.email);
    assert_eq!(driver.gender, original.gender);
    assert_eq!(driver.language, original.language);
    assert_eq!(driver.phone_number, original.phone_number);
    assert!(driver.verified_at.is_some(), "verified_at must remain set");
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_update_driver_info_firstname_formatting(ctx: &mut context::TestContext) {
    let original = ctx
        .repositories
        .driver_database_repository
        .get_driver_by_id(ctx.authenticated_user_id)
        .await
        .unwrap()
        .unwrap();

    let res = ctx
        .authenticated_router
        .patch("/me")
        .json(&json!({ "firstname": "jean-luc" }))
        .await;

    res.assert_status(StatusCode::OK);

    let driver = ctx
        .repositories
        .driver_database_repository
        .get_driver_by_id(ctx.authenticated_user_id)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(
        driver.firstname, "Jean-Luc",
        "Hyphenated firstname must be title-cased"
    );

    ctx.repositories
        .driver_database_repository
        .update_driver(original)
        .await
        .unwrap();
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_update_driver_info_lastname_formatting(ctx: &mut context::TestContext) {
    let original = ctx
        .repositories
        .driver_database_repository
        .get_driver_by_id(ctx.authenticated_user_id)
        .await
        .unwrap()
        .unwrap();

    let res = ctx
        .authenticated_router
        .patch("/me")
        .json(&json!({ "lastname": "de la montagne" }))
        .await;

    res.assert_status(StatusCode::OK);

    let driver = ctx
        .repositories
        .driver_database_repository
        .get_driver_by_id(ctx.authenticated_user_id)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(
        driver.lastname, "De La Montagne",
        "Multi-word lastname must be title-cased"
    );

    ctx.repositories
        .driver_database_repository
        .update_driver(original)
        .await
        .unwrap();
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_update_driver_info_gender_formatting(ctx: &mut context::TestContext) {
    let original = ctx
        .repositories
        .driver_database_repository
        .get_driver_by_id(ctx.authenticated_user_id)
        .await
        .unwrap()
        .unwrap();

    let res = ctx
        .authenticated_router
        .patch("/me")
        .json(&json!({ "gender": "f" }))
        .await;

    res.assert_status(StatusCode::OK);

    let driver = ctx
        .repositories
        .driver_database_repository
        .get_driver_by_id(ctx.authenticated_user_id)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(
        driver.gender,
        Some("F".to_string()),
        "Gender must be uppercased"
    );

    ctx.repositories
        .driver_database_repository
        .update_driver(original)
        .await
        .unwrap();
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_update_driver_info_email_normalization(ctx: &mut context::TestContext) {
    let original = ctx
        .repositories
        .driver_database_repository
        .get_driver_by_id(ctx.authenticated_user_id)
        .await
        .unwrap()
        .unwrap();

    let res = ctx
        .authenticated_router
        .patch("/me")
        .json(&json!({ "email": "NEW.Email@EXAMPLE.BE" }))
        .await;

    res.assert_status(StatusCode::OK);

    let driver = ctx
        .repositories
        .driver_database_repository
        .get_driver_by_id(ctx.authenticated_user_id)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(
        driver.email, "new.email@example.be",
        "Email must be normalized to lowercase"
    );

    ctx.repositories
        .driver_database_repository
        .update_driver(original)
        .await
        .unwrap();
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_update_driver_info_same_email_stays_verified(ctx: &mut context::TestContext) {
    let original = ctx
        .repositories
        .driver_database_repository
        .get_driver_by_id(ctx.authenticated_user_id)
        .await
        .unwrap()
        .unwrap();

    assert!(
        original.verified_at.is_some(),
        "Test user must start as verified"
    );

    // Sending the same email (possibly with different casing) must not reset verification
    let res = ctx
        .authenticated_router
        .patch("/me")
        .json(&json!({ "email": "TEST.USER@EXAMPLE.BE" }))
        .await;

    res.assert_status(StatusCode::OK);

    let driver = ctx
        .repositories
        .driver_database_repository
        .get_driver_by_id(ctx.authenticated_user_id)
        .await
        .unwrap()
        .unwrap();

    assert!(
        driver.verified_at.is_some(),
        "verified_at must remain set when the (normalised) email does not change"
    );
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_update_driver_info_email_change_clears_verification(ctx: &mut context::TestContext) {
    let original = ctx
        .repositories
        .driver_database_repository
        .get_driver_by_id(ctx.authenticated_user_id)
        .await
        .unwrap()
        .unwrap();

    assert!(
        original.verified_at.is_some(),
        "Test user must start as verified"
    );

    let res = ctx
        .authenticated_router
        .patch("/me")
        .json(&json!({ "email": "changed.email@example.be" }))
        .await;

    res.assert_status(StatusCode::OK);
    let body: CreateDriverResponse = res.json();
    assert!(!body.access_token.is_empty());

    // DB state: verified_at must be cleared and email normalised
    let driver = ctx
        .repositories
        .driver_database_repository
        .get_driver_by_id(ctx.authenticated_user_id)
        .await
        .unwrap()
        .unwrap();

    assert!(
        driver.verified_at.is_none(),
        "verified_at must be cleared after email change"
    );
    assert_eq!(driver.email, "changed.email@example.be");

    // The new access token must embed verified = false
    let claims = decode_access_token(&body.access_token);
    assert!(
        !claims.driver.verified,
        "New token must carry verified=false after email change"
    );
    assert_eq!(claims.driver.email, "changed.email@example.be");

    ctx.repositories
        .driver_database_repository
        .update_driver(original)
        .await
        .unwrap();
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_update_driver_info_email_change_new_token_is_rejected(
    ctx: &mut context::TestContext,
) {
    let original = ctx
        .repositories
        .driver_database_repository
        .get_driver_by_id(ctx.authenticated_user_id)
        .await
        .unwrap()
        .unwrap();

    let res = ctx
        .authenticated_router
        .patch("/me")
        .json(&json!({ "email": "changed.email@example.be" }))
        .await;

    res.assert_status(StatusCode::OK);
    let body: CreateDriverResponse = res.json();

    // A protected route must reject the new (unverified) token with DRIVER_NOT_VERIFIED
    let mut router_with_new_token = axum_test::TestServer::new(ctx.app.app_router()).unwrap();
    router_with_new_token.add_header(
        axum::http::header::AUTHORIZATION,
        format!("Bearer {}", body.access_token),
    );

    let get_res = router_with_new_token.get("/rest-periods").await;
    get_res.assert_status(StatusCode::UNAUTHORIZED);
    let get_body: ErrorBody = get_res.json();
    assert_eq!(get_body.error_code, "DRIVER_NOT_VERIFIED");

    ctx.repositories
        .driver_database_repository
        .update_driver(original)
        .await
        .unwrap();
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_update_driver_info_email_already_taken(ctx: &mut context::TestContext) {
    // "test-bis.user@example.be" is the email of the second driver in the test dataset
    let res = ctx
        .authenticated_router
        .patch("/me")
        .json(&json!({ "email": "test-bis.user@example.be" }))
        .await;

    res.assert_status(StatusCode::CONFLICT);
    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "DRIVER_ALREADY_EXISTS");
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_update_driver_info_email_domain_denylisted(ctx: &mut context::TestContext) {
    // "example.fr" and "example.com" are in the denylist (see TestContext config)
    let res1 = ctx
        .authenticated_router
        .patch("/me")
        .json(&json!({ "email": "new.email@example.fr" }))
        .await;

    res1.assert_status(StatusCode::BAD_REQUEST);
    let body1: ErrorBody = res1.json();
    assert_eq!(body1.error_code, "EMAIL_DOMAIN_DENYLISTED");
    assert_eq!(
        body1
            .content
            .as_ref()
            .and_then(|c| c.get("domain"))
            .and_then(|v| v.as_str()),
        Some("example.fr")
    );

    let res2 = ctx
        .authenticated_router
        .patch("/me")
        .json(&json!({ "email": "new.email@EXAMPLE.COM" }))
        .await;

    res2.assert_status(StatusCode::BAD_REQUEST);
    let body2: ErrorBody = res2.json();
    assert_eq!(body2.error_code, "EMAIL_DOMAIN_DENYLISTED");
    assert_eq!(
        body2
            .content
            .as_ref()
            .and_then(|c| c.get("domain"))
            .and_then(|v| v.as_str()),
        Some("example.com")
    );
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_update_driver_info_email_denylist_not_checked_for_unchanged_email(
    ctx: &mut context::TestContext,
) {
    // The current email domain ("example.be") is not denylisted. This test confirms that
    // omitting the email field entirely bypasses any denylist logic — no 400 is returned.
    let res = ctx
        .authenticated_router
        .patch("/me")
        .json(&json!({ "firstname": "Jean" }))
        .await;

    res.assert_status(StatusCode::OK);

    // Restore firstname
    let original = ctx
        .repositories
        .driver_database_repository
        .get_driver_by_id(ctx.authenticated_user_id)
        .await
        .unwrap()
        .unwrap();
    ctx.repositories
        .driver_database_repository
        .update_driver({
            let mut d = original;
            d.firstname = "Test".to_string();
            d
        })
        .await
        .unwrap();
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_update_driver_info_password_change(ctx: &mut context::TestContext) {
    let original = ctx
        .repositories
        .driver_database_repository
        .get_driver_by_id(ctx.authenticated_user_id)
        .await
        .unwrap()
        .unwrap();

    let res = ctx
        .authenticated_router
        .patch("/me")
        .json(&json!({ "password": "NewSecurePass456!" }))
        .await;

    res.assert_status(StatusCode::OK);

    // The new password must allow login
    let login_res = ctx
        .unauthenticated_router
        .post("/authentication/login")
        .json(&json!({
            "email": "test.user@example.be",
            "password": "NewSecurePass456!"
        }))
        .await;
    login_res.assert_status(StatusCode::OK);

    // The old password must no longer work
    let old_login_res = ctx
        .unauthenticated_router
        .post("/authentication/login")
        .json(&json!({
            "email": "test.user@example.be",
            "password": "Baptiste01!"
        }))
        .await;
    old_login_res.assert_status(StatusCode::UNAUTHORIZED);

    ctx.repositories
        .driver_database_repository
        .update_driver(original)
        .await
        .unwrap();
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_update_driver_info_phone_number_valid(ctx: &mut context::TestContext) {
    let original = ctx
        .repositories
        .driver_database_repository
        .get_driver_by_id(ctx.authenticated_user_id)
        .await
        .unwrap()
        .unwrap();

    let res = ctx
        .authenticated_router
        .patch("/me")
        .json(&json!({ "phone_number": "+32476123456" }))
        .await;

    res.assert_status(StatusCode::OK);

    let driver = ctx
        .repositories
        .driver_database_repository
        .get_driver_by_id(ctx.authenticated_user_id)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(
        driver.phone_number,
        Some("+32476123456".to_string()),
        "Phone number must be stored as provided"
    );

    ctx.repositories
        .driver_database_repository
        .update_driver(original)
        .await
        .unwrap();
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_update_driver_info_phone_number_invalid_format(ctx: &mut context::TestContext) {
    // Missing leading '+'
    let res1 = ctx
        .authenticated_router
        .patch("/me")
        .json(&json!({ "phone_number": "0476123456" }))
        .await;
    res1.assert_status(StatusCode::BAD_REQUEST);
    assert_eq!(res1.json::<ErrorBody>().error_code, "BODY_VALIDATION");

    // Country code starts with 0
    let res2 = ctx
        .authenticated_router
        .patch("/me")
        .json(&json!({ "phone_number": "+0476123456" }))
        .await;
    res2.assert_status(StatusCode::BAD_REQUEST);
    assert_eq!(res2.json::<ErrorBody>().error_code, "BODY_VALIDATION");

    // Too short (less than 7 digits)
    let res3 = ctx
        .authenticated_router
        .patch("/me")
        .json(&json!({ "phone_number": "+12345" }))
        .await;
    res3.assert_status(StatusCode::BAD_REQUEST);
    assert_eq!(res3.json::<ErrorBody>().error_code, "BODY_VALIDATION");

    // Too long (more than 15 digits)
    let res4 = ctx
        .authenticated_router
        .patch("/me")
        .json(&json!({ "phone_number": "+1234567890123456" }))
        .await;
    res4.assert_status(StatusCode::BAD_REQUEST);
    assert_eq!(res4.json::<ErrorBody>().error_code, "BODY_VALIDATION");

    // Non-digit characters after '+'
    let res5 = ctx
        .authenticated_router
        .patch("/me")
        .json(&json!({ "phone_number": "+33abc123456" }))
        .await;
    res5.assert_status(StatusCode::BAD_REQUEST);
    assert_eq!(res5.json::<ErrorBody>().error_code, "BODY_VALIDATION");
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_update_driver_info_language_change(ctx: &mut context::TestContext) {
    let original = ctx
        .repositories
        .driver_database_repository
        .get_driver_by_id(ctx.authenticated_user_id)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(
        original.language, "fr",
        "Test user must start with language 'fr'"
    );

    let res = ctx
        .authenticated_router
        .patch("/me")
        .json(&json!({ "language": "en" }))
        .await;

    res.assert_status(StatusCode::OK);

    let driver = ctx
        .repositories
        .driver_database_repository
        .get_driver_by_id(ctx.authenticated_user_id)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(driver.language, "en", "Language must be updated to 'en'");

    ctx.repositories
        .driver_database_repository
        .update_driver(original)
        .await
        .unwrap();
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_update_driver_info_validation_empty_strings(ctx: &mut context::TestContext) {
    // Empty firstname
    let res1 = ctx
        .authenticated_router
        .patch("/me")
        .json(&json!({ "firstname": "" }))
        .await;
    res1.assert_status(StatusCode::BAD_REQUEST);
    assert_eq!(res1.json::<ErrorBody>().error_code, "BODY_VALIDATION");

    // Empty lastname
    let res2 = ctx
        .authenticated_router
        .patch("/me")
        .json(&json!({ "lastname": "" }))
        .await;
    res2.assert_status(StatusCode::BAD_REQUEST);
    assert_eq!(res2.json::<ErrorBody>().error_code, "BODY_VALIDATION");

    // Empty email
    let res3 = ctx
        .authenticated_router
        .patch("/me")
        .json(&json!({ "email": "" }))
        .await;
    res3.assert_status(StatusCode::BAD_REQUEST);
    assert_eq!(res3.json::<ErrorBody>().error_code, "BODY_VALIDATION");
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_update_driver_info_validation_overflow(ctx: &mut context::TestContext) {
    let long_name = "a".repeat(256);
    let long_password = "a".repeat(41);

    // Firstname too long
    let res1 = ctx
        .authenticated_router
        .patch("/me")
        .json(&json!({ "firstname": long_name }))
        .await;
    res1.assert_status(StatusCode::BAD_REQUEST);
    assert_eq!(res1.json::<ErrorBody>().error_code, "BODY_VALIDATION");

    // Password too long
    let res2 = ctx
        .authenticated_router
        .patch("/me")
        .json(&json!({ "password": long_password }))
        .await;
    res2.assert_status(StatusCode::BAD_REQUEST);
    assert_eq!(res2.json::<ErrorBody>().error_code, "BODY_VALIDATION");
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_update_driver_info_validation_gender_invalid(ctx: &mut context::TestContext) {
    // Gender must be exactly one character
    let res1 = ctx
        .authenticated_router
        .patch("/me")
        .json(&json!({ "gender": "MF" }))
        .await;
    res1.assert_status(StatusCode::BAD_REQUEST);
    assert_eq!(res1.json::<ErrorBody>().error_code, "BODY_VALIDATION");

    let res2 = ctx
        .authenticated_router
        .patch("/me")
        .json(&json!({ "gender": "" }))
        .await;
    res2.assert_status(StatusCode::BAD_REQUEST);
    assert_eq!(res2.json::<ErrorBody>().error_code, "BODY_VALIDATION");
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_update_driver_info_validation_email_invalid(ctx: &mut context::TestContext) {
    let res = ctx
        .authenticated_router
        .patch("/me")
        .json(&json!({ "email": "not-an-email" }))
        .await;

    res.assert_status(StatusCode::BAD_REQUEST);
    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "BODY_VALIDATION");
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_update_driver_info_validation_password_too_short(ctx: &mut context::TestContext) {
    let res = ctx
        .authenticated_router
        .patch("/me")
        .json(&json!({ "password": "short" }))
        .await;

    res.assert_status(StatusCode::BAD_REQUEST);
    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "BODY_VALIDATION");
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_update_driver_info_new_token_reflects_updated_claims(ctx: &mut context::TestContext) {
    let original = ctx
        .repositories
        .driver_database_repository
        .get_driver_by_id(ctx.authenticated_user_id)
        .await
        .unwrap()
        .unwrap();

    let res = ctx
        .authenticated_router
        .patch("/me")
        .json(&json!({
            "firstname": "updated",
            "lastname": "DRIVER"
        }))
        .await;

    res.assert_status(StatusCode::OK);
    let body: CreateDriverResponse = res.json();

    let claims = decode_access_token(&body.access_token);
    assert_eq!(claims.driver.first_name, "Updated");
    assert_eq!(claims.driver.last_name, "Driver");
    assert!(
        claims.driver.verified,
        "Verified status must remain true when email did not change"
    );

    ctx.repositories
        .driver_database_repository
        .update_driver(original)
        .await
        .unwrap();
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_deactivate_driver_unauthorized(ctx: &mut context::TestContext) {
    let res = ctx.unauthenticated_router.post("/me/deactivate").await;

    res.assert_status(StatusCode::UNAUTHORIZED);
    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "UNAUTHORIZED");
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_reactivate_driver_unauthorized(ctx: &mut context::TestContext) {
    let res = ctx.unauthenticated_router.post("/me/reactivate").await;

    res.assert_status(StatusCode::UNAUTHORIZED);
    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "UNAUTHORIZED");
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_deactivate_driver_success(ctx: &mut context::TestContext) {
    let original = ctx
        .repositories
        .driver_database_repository
        .get_driver_by_id(ctx.authenticated_user_id)
        .await
        .unwrap()
        .unwrap();

    assert!(
        original.deactivated_at.is_none(),
        "Test user must start as active"
    );

    let before = chrono::Utc::now();
    let res = ctx.authenticated_router.post("/me/deactivate").await;
    res.assert_status(StatusCode::OK);
    let after = chrono::Utc::now();

    let driver = ctx
        .repositories
        .driver_database_repository
        .get_driver_by_id(ctx.authenticated_user_id)
        .await
        .unwrap()
        .unwrap();

    let deactivated_at = driver
        .deactivated_at
        .expect("deactivated_at must be set after deactivation");

    // deactivated_at must be Utc::now() + 30 days (ACCOUNT_DEACTIVATION_DAYS default)
    let expected_min = before + chrono::Duration::days(30);
    let expected_max = after + chrono::Duration::days(30);
    assert!(
        deactivated_at >= expected_min && deactivated_at <= expected_max,
        "deactivated_at ({}) must be within [{}, {}]",
        deactivated_at,
        expected_min,
        expected_max
    );

    ctx.repositories
        .driver_database_repository
        .update_driver(original)
        .await
        .unwrap();
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_deactivate_driver_already_deactivated(ctx: &mut context::TestContext) {
    let original = ctx
        .repositories
        .driver_database_repository
        .get_driver_by_id(ctx.authenticated_user_id)
        .await
        .unwrap()
        .unwrap();

    // First deactivation must succeed
    ctx.authenticated_router
        .post("/me/deactivate")
        .await
        .assert_status(StatusCode::OK);

    // Second deactivation must be rejected
    let res = ctx.authenticated_router.post("/me/deactivate").await;
    res.assert_status(StatusCode::CONFLICT);
    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "ACCOUNT_ALREADY_DEACTIVATED");

    ctx.repositories
        .driver_database_repository
        .update_driver(original)
        .await
        .unwrap();
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_reactivate_driver_success(ctx: &mut context::TestContext) {
    let original = ctx
        .repositories
        .driver_database_repository
        .get_driver_by_id(ctx.authenticated_user_id)
        .await
        .unwrap()
        .unwrap();

    // Deactivate first
    ctx.authenticated_router
        .post("/me/deactivate")
        .await
        .assert_status(StatusCode::OK);

    // Then reactivate
    let res = ctx.authenticated_router.post("/me/reactivate").await;
    res.assert_status(StatusCode::OK);

    let driver = ctx
        .repositories
        .driver_database_repository
        .get_driver_by_id(ctx.authenticated_user_id)
        .await
        .unwrap()
        .unwrap();

    assert!(
        driver.deactivated_at.is_none(),
        "deactivated_at must be cleared after reactivation"
    );

    ctx.repositories
        .driver_database_repository
        .update_driver(original)
        .await
        .unwrap();
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_reactivate_driver_not_deactivated(ctx: &mut context::TestContext) {
    // Account is active — reactivation must be rejected
    let res = ctx.authenticated_router.post("/me/reactivate").await;

    res.assert_status(StatusCode::CONFLICT);
    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "ACCOUNT_NOT_DEACTIVATED");
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_deactivate_then_reactivate_full_cycle(ctx: &mut context::TestContext) {
    let original = ctx
        .repositories
        .driver_database_repository
        .get_driver_by_id(ctx.authenticated_user_id)
        .await
        .unwrap()
        .unwrap();

    // Deactivate
    ctx.authenticated_router
        .post("/me/deactivate")
        .await
        .assert_status(StatusCode::OK);

    let after_deactivation = ctx
        .repositories
        .driver_database_repository
        .get_driver_by_id(ctx.authenticated_user_id)
        .await
        .unwrap()
        .unwrap();
    assert!(after_deactivation.deactivated_at.is_some());

    // Reactivate
    ctx.authenticated_router
        .post("/me/reactivate")
        .await
        .assert_status(StatusCode::OK);

    let after_reactivation = ctx
        .repositories
        .driver_database_repository
        .get_driver_by_id(ctx.authenticated_user_id)
        .await
        .unwrap()
        .unwrap();
    assert!(after_reactivation.deactivated_at.is_none());

    // A second deactivation must work (account is active again)
    ctx.authenticated_router
        .post("/me/deactivate")
        .await
        .assert_status(StatusCode::OK);

    ctx.repositories
        .driver_database_repository
        .update_driver(original)
        .await
        .unwrap();
}
