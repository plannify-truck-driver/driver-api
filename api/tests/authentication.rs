use api::http::common::api_error::ErrorBody;
use plannify_driver_api_core::domain::{
    driver::{
        entities::{CreateDriverResponse, DriverLimitationRow, DriverSuspensionRow, EntityType},
        port::{DriverCacheKeyType, DriverCacheRepository, DriverDatabaseRepository},
    },
    employee::port::EmployeeRepository,
};
use reqwest::StatusCode;
use serde_json::json;
use serial_test::serial;
use test_context::test_context;
use uuid::Uuid;

pub mod context;
pub mod helpers;

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_signup_success(ctx: &mut context::TestContext) {
    let res = ctx
        .unauthenticated_router
        .post("/authentication/signup")
        .json(&json!({
            "firstname": "john",
            "lastname": "DOE",
            "gender": null,
            "email": "john.DOE@mAIL.coM",
            "password": "securePassword123",
            "language": "fr"
        }))
        .await;

    res.assert_status(StatusCode::CREATED);
    let body: CreateDriverResponse = res.json();
    assert!(!body.access_token.is_empty());

    let driver = ctx
        .repositories
        .driver_database_repository
        .get_driver_by_email("john.doe@mail.com".to_string())
        .await
        .unwrap();

    assert_eq!(driver.firstname, "John", "Firstname should be capitalized");
    assert_eq!(driver.lastname, "Doe", "Lastname should be capitalized");
    assert_eq!(driver.gender, None, "Gender should be None");
    assert_eq!(
        driver.email, "john.doe@mail.com",
        "Email should be normalized to lowercase"
    );

    ctx.repositories
        .driver_database_repository
        .delete_driver(driver.pk_driver_id)
        .await
        .unwrap();
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_signup_with_banned_email(ctx: &mut context::TestContext) {
    let res1 = ctx
        .unauthenticated_router
        .post("/authentication/signup")
        .json(&json!({
            "firstname": "John",
            "lastname": "Doe",
            "gender": null,
            "email": "john.doe@example.com",
            "password": "securePassword123",
            "language": "fr"
        }))
        .await;

    res1.assert_status(StatusCode::BAD_REQUEST);
    let body1: ErrorBody = res1.json();

    assert_eq!(body1.error_code, "EMAIL_DOMAIN_DENYLISTED");

    let res2 = ctx
        .unauthenticated_router
        .post("/authentication/signup")
        .json(&json!({
            "firstname": "John",
            "lastname": "Doe",
            "gender": null,
            "email": "john.DOE@ExaMPle.com",
            "password": "securePassword123",
            "language": "fr"
        }))
        .await;

    res2.assert_status(StatusCode::BAD_REQUEST);
    let body2: ErrorBody = res2.json();

    assert_eq!(body2.error_code, "EMAIL_DOMAIN_DENYLISTED");
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_signup_with_wrong_body(ctx: &mut context::TestContext) {
    let res1 = ctx
        .unauthenticated_router
        .post("/authentication/signup")
        .json(&json!({
            "firstname": "",
            "lastname": "",
            "gender": "",
            "email": "",
            "password": "",
            "language": "fr"
        }))
        .await;

    res1.assert_status(StatusCode::BAD_REQUEST);
    let body1: ErrorBody = res1.json();

    assert_eq!(body1.error_code, "BODY_VALIDATION");

    let content_body1 = body1.content.as_ref().unwrap();
    assert!(content_body1.is_mapping());
    assert_eq!(content_body1.as_mapping().unwrap().len(), 5);

    let res2 = ctx
        .unauthenticated_router
        .post("/authentication/signup")
        .json(&json!({
            "firstname": "firstnamefirstnamefirstnamefirstnamefirstnamefirstnamefirstnamefirstnamefirstnamefirstnamefirstnamefirstnamefirstnamefirstnamefirstnamefirstnamefirstnamefirstnamefirstnamefirstnamefirstnamefirstnamefirstnamefirstnamefirstnamefirstnamefirstnamefirstnamefirstname",
            "lastname": "lastnamelastnamelastnamelastnamelastnamelastnamelastnamelastnamelastnamelastnamelastnamelastnamelastnamelastnamelastnamelastnamelastnamelastnamelastnamelastnamelastnamelastnamelastnamelastnamelastnamelastnamelastnamelastnamelastnamelastnamelastnamelastname",
            "gender": "MF",
            "email": "firstname.lastname@mail.comfirstname.lastname@mail.comfirstname.lastname@mail.comfirstname.lastname@mail.comfirstname.lastname@mail.comfirstname.lastname@mail.comfirstname.lastname@mail.comfirstname.lastname@mail.comfirstname.lastname@mail.comfirstname.lastname@mail.com",
            "password": "passwordpasswordpasswordpasswordpasswordpassword",
            "language": "fr"
        }))
        .await;

    res2.assert_status(StatusCode::BAD_REQUEST);
    let body2: ErrorBody = res2.json();

    assert_eq!(body2.error_code, "BODY_VALIDATION");

    let content_body2 = body2.content.as_ref().unwrap();
    assert!(content_body2.is_mapping());
    assert_eq!(content_body2.as_mapping().unwrap().len(), 5);
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_signup_already_exists(ctx: &mut context::TestContext) {
    let res = ctx
        .unauthenticated_router
        .post("/authentication/signup")
        .json(&json!({
            "firstname": "John",
            "lastname": "Doe",
            "gender": null,
            "email": "TeST.user@ExAMplE.Be",
            "password": "securePassword123",
            "language": "fr"
        }))
        .await;

    res.assert_status(StatusCode::CONFLICT);
    let body: ErrorBody = res.json();

    assert_eq!(body.error_code, "DRIVER_ALREADY_EXISTS");
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_signup_with_entity_limitations(ctx: &mut context::TestContext) {
    let employee = ctx
        .repositories
        .employee_repository
        .get_first_employee()
        .await
        .unwrap();

    if employee.is_none() {
        panic!("No employee found in the database");
    }
    let employee = employee.unwrap();

    let limitation = DriverLimitationRow {
        pk_maximum_entity_limit_id: 100,
        entity_type: EntityType::DRIVER,
        maximum_limit: 0,
        fk_created_employee_id: employee.pk_employee_id,
        start_at: chrono::Utc::now(),
        end_at: None,
        created_at: chrono::Utc::now(),
    };
    let result = ctx
        .repositories
        .driver_database_repository
        .create_driver_limitation(limitation)
        .await;

    if let Err(e) = &result {
        eprintln!("Error creating driver limitation: {:?}", e);
        panic!("Failed to create limitation: {:?}", e);
    }

    let result = result.unwrap();

    let res = ctx
        .unauthenticated_router
        .post("/authentication/signup")
        .json(&json!({
            "firstname": "John",
            "lastname": "Doe",
            "gender": null,
            "email": "test.user@example.be",
            "password": "securePassword123",
            "language": "fr"
        }))
        .await;

    res.assert_status(StatusCode::BAD_REQUEST);
    let body: ErrorBody = res.json();

    assert_eq!(body.error_code, "DRIVER_LIMIT_REACHED");

    ctx.repositories
        .driver_database_repository
        .delete_driver_limitation(result.pk_maximum_entity_limit_id)
        .await
        .unwrap();
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_signup_with_entity_limitations_outbound(ctx: &mut context::TestContext) {
    let employee = ctx
        .repositories
        .employee_repository
        .get_first_employee()
        .await
        .unwrap();

    if employee.is_none() {
        panic!("No employee found in the database");
    }
    let employee = employee.unwrap();

    let now = chrono::Utc::now();
    let limitation_before = DriverLimitationRow {
        pk_maximum_entity_limit_id: 1,
        entity_type: EntityType::DRIVER,
        maximum_limit: 0,
        fk_created_employee_id: employee.pk_employee_id,
        start_at: now - chrono::Duration::days(30),
        end_at: Some(now - chrono::Duration::days(1)),
        created_at: now,
    };
    let limitation_after = DriverLimitationRow {
        pk_maximum_entity_limit_id: 2,
        entity_type: EntityType::DRIVER,
        maximum_limit: 0,
        fk_created_employee_id: employee.pk_employee_id,
        start_at: now + chrono::Duration::days(1),
        end_at: Some(now + chrono::Duration::days(30)),
        created_at: now,
    };
    let result_before = ctx
        .repositories
        .driver_database_repository
        .create_driver_limitation(limitation_before)
        .await
        .unwrap();
    let result_after = ctx
        .repositories
        .driver_database_repository
        .create_driver_limitation(limitation_after)
        .await
        .unwrap();

    let res = ctx
        .unauthenticated_router
        .post("/authentication/signup")
        .json(&json!({
            "firstname": "John",
            "lastname": "Doe",
            "gender": null,
            "email": "test.user@example.be",
            "password": "securePassword123",
            "language": "fr"
        }))
        .await;

    res.assert_status(StatusCode::CONFLICT);
    let body: ErrorBody = res.json();

    assert_eq!(body.error_code, "DRIVER_ALREADY_EXISTS");

    ctx.repositories
        .driver_database_repository
        .delete_driver_limitation(result_before.pk_maximum_entity_limit_id)
        .await
        .unwrap();
    ctx.repositories
        .driver_database_repository
        .delete_driver_limitation(result_after.pk_maximum_entity_limit_id)
        .await
        .unwrap();
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_signup_with_entity_limitations_other_entity(ctx: &mut context::TestContext) {
    let employee = ctx
        .repositories
        .employee_repository
        .get_first_employee()
        .await
        .unwrap();

    if employee.is_none() {
        panic!("No employee found in the database");
    }
    let employee = employee.unwrap();

    let limitation = DriverLimitationRow {
        pk_maximum_entity_limit_id: 0,
        entity_type: EntityType::EMPLOYEE,
        maximum_limit: 0,
        fk_created_employee_id: employee.pk_employee_id,
        start_at: chrono::Utc::now(),
        end_at: None,
        created_at: chrono::Utc::now(),
    };
    let result = ctx
        .repositories
        .driver_database_repository
        .create_driver_limitation(limitation)
        .await;

    if let Err(e) = &result {
        eprintln!("Error creating driver limitation: {:?}", e);
        panic!("Failed to create limitation: {:?}", e);
    }

    let result = result.unwrap();

    let res = ctx
        .unauthenticated_router
        .post("/authentication/signup")
        .json(&json!({
            "firstname": "John",
            "lastname": "Doe",
            "gender": null,
            "email": "test.user@example.be",
            "password": "securePassword123",
            "language": "fr"
        }))
        .await;

    res.assert_status(StatusCode::CONFLICT);
    let body: ErrorBody = res.json();

    assert_eq!(body.error_code, "DRIVER_ALREADY_EXISTS");

    ctx.repositories
        .driver_database_repository
        .delete_driver_limitation(result.pk_maximum_entity_limit_id)
        .await
        .unwrap();
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_login_success(ctx: &mut context::TestContext) {
    let res = ctx
        .unauthenticated_router
        .post("/authentication/login")
        .json(&json!({
            "email": "TeST.usEr@eXAmPlE.be",
            "password": "Baptiste01!"
        }))
        .await;

    res.assert_status(StatusCode::OK);
    let body: CreateDriverResponse = res.json();
    assert!(!body.access_token.is_empty());
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_login_with_wrong_email(ctx: &mut context::TestContext) {
    let res = ctx
        .unauthenticated_router
        .post("/authentication/login")
        .json(&json!({
            "email": "TeST.usEr@eXAmPlE.com",
            "password": "Baptiste01!"
        }))
        .await;

    res.assert_status(StatusCode::UNAUTHORIZED);
    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "INVALID_CREDENTIALS");
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_login_with_wrong_password(ctx: &mut context::TestContext) {
    let res = ctx
        .unauthenticated_router
        .post("/authentication/login")
        .json(&json!({
            "email": "TeST.usEr@eXAmPlE.be",
            "password": "Baptiste01!wrong"
        }))
        .await;

    res.assert_status(StatusCode::UNAUTHORIZED);
    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "INVALID_CREDENTIALS");
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_login_with_suspension(ctx: &mut context::TestContext) {
    let employee = ctx
        .repositories
        .employee_repository
        .get_first_employee()
        .await
        .unwrap();

    if employee.is_none() {
        panic!("No employee found in the database");
    }
    let employee = employee.unwrap();

    let suspension = ctx
        .repositories
        .driver_database_repository
        .create_driver_suspension(DriverSuspensionRow {
            pk_driver_suspension_id: 0,
            fk_driver_id: ctx.authenticated_user_id,
            fk_created_employee_id: employee.pk_employee_id,
            driver_message: Some("Test suspension".to_string()),
            title: "Test Suspension".to_string(),
            description: Some("This is a test suspension".to_string()),
            start_at: chrono::Utc::now() - chrono::Duration::hours(1),
            end_at: Some(chrono::Utc::now() + chrono::Duration::hours(1)),
            can_access_restricted_space: false,
            created_at: chrono::Utc::now(),
        })
        .await
        .unwrap();

    let res = ctx
        .unauthenticated_router
        .post("/authentication/login")
        .json(&json!({
            "email": "test.user@example.be",
            "password": "Baptiste01!"
        }))
        .await;

    res.assert_status(StatusCode::FORBIDDEN);
    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "DRIVER_SUSPENDED");

    ctx.repositories
        .driver_database_repository
        .delete_driver_suspension(suspension.pk_driver_suspension_id)
        .await
        .unwrap();
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_login_with_suspension_outbound(ctx: &mut context::TestContext) {
    let employee = ctx
        .repositories
        .employee_repository
        .get_first_employee()
        .await
        .unwrap();

    if employee.is_none() {
        panic!("No employee found in the database");
    }
    let employee = employee.unwrap();

    let suspension = ctx
        .repositories
        .driver_database_repository
        .create_driver_suspension(DriverSuspensionRow {
            pk_driver_suspension_id: 0,
            fk_driver_id: ctx.authenticated_user_id,
            fk_created_employee_id: employee.pk_employee_id,
            driver_message: Some("Test suspension".to_string()),
            title: "Test Suspension".to_string(),
            description: Some("This is a test suspension".to_string()),
            start_at: chrono::Utc::now() - chrono::Duration::days(30),
            end_at: Some(chrono::Utc::now() - chrono::Duration::days(1)),
            can_access_restricted_space: false,
            created_at: chrono::Utc::now(),
        })
        .await
        .unwrap();

    let res = ctx
        .unauthenticated_router
        .post("/authentication/login")
        .json(&json!({
            "email": "test.user@example.be",
            "password": "Baptiste01!"
        }))
        .await;

    res.assert_status(StatusCode::OK);
    let body: CreateDriverResponse = res.json();
    assert!(!body.access_token.is_empty());

    ctx.repositories
        .driver_database_repository
        .delete_driver_suspension(suspension.pk_driver_suspension_id)
        .await
        .unwrap();
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_login_with_suspension_can_access_restricted_space(ctx: &mut context::TestContext) {
    let employee = ctx
        .repositories
        .employee_repository
        .get_first_employee()
        .await
        .unwrap();

    if employee.is_none() {
        panic!("No employee found in the database");
    }
    let employee = employee.unwrap();

    let suspension = ctx
        .repositories
        .driver_database_repository
        .create_driver_suspension(DriverSuspensionRow {
            pk_driver_suspension_id: 0,
            fk_driver_id: ctx.authenticated_user_id,
            fk_created_employee_id: employee.pk_employee_id,
            driver_message: Some("Test suspension".to_string()),
            title: "Test Suspension".to_string(),
            description: Some("This is a test suspension".to_string()),
            start_at: chrono::Utc::now() - chrono::Duration::days(1),
            end_at: Some(chrono::Utc::now() + chrono::Duration::days(1)),
            can_access_restricted_space: true,
            created_at: chrono::Utc::now(),
        })
        .await
        .unwrap();

    let res = ctx
        .unauthenticated_router
        .post("/authentication/login")
        .json(&json!({
            "email": "test.user@example.be",
            "password": "Baptiste01!"
        }))
        .await;

    res.assert_status(StatusCode::OK);
    let body: CreateDriverResponse = res.json();
    assert!(!body.access_token.is_empty());

    ctx.repositories
        .driver_database_repository
        .delete_driver_suspension(suspension.pk_driver_suspension_id)
        .await
        .unwrap();
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_request_password_reset_success(ctx: &mut context::TestContext) {
    sqlx::query(
        "UPDATE drivers SET mail_preferences = mail_preferences | 2 WHERE pk_driver_id = $1",
    )
    .bind(ctx.authenticated_user_id)
    .execute(&ctx.repositories.pool)
    .await
    .unwrap();

    let res = ctx
        .unauthenticated_router
        .post("/authentication/reset-password")
        .json(&json!({ "email": "TeST.uSeR@eXaMpLe.Be" }))
        .await;

    res.assert_status(StatusCode::OK);

    let driver = ctx
        .repositories
        .driver_database_repository
        .get_driver_by_email("test.user@example.be".to_string())
        .await
        .unwrap();

    let (redis_key, _) = ctx
        .repositories
        .driver_cache_repository
        .get_key_by_type(driver.pk_driver_id, DriverCacheKeyType::ResetPassword);
    let token = ctx
        .repositories
        .driver_cache_repository
        .get_redis(redis_key)
        .await
        .unwrap();

    assert!(
        token.is_some(),
        "Reset password token should be stored in Redis"
    );

    sqlx::query(
        "UPDATE drivers SET mail_preferences = mail_preferences & ~2 WHERE pk_driver_id = $1",
    )
    .bind(ctx.authenticated_user_id)
    .execute(&ctx.repositories.pool)
    .await
    .unwrap();
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_request_password_reset_token_already_exists(ctx: &mut context::TestContext) {
    sqlx::query(
        "UPDATE drivers SET mail_preferences = mail_preferences | 2 WHERE pk_driver_id = $1",
    )
    .bind(ctx.authenticated_user_id)
    .execute(&ctx.repositories.pool)
    .await
    .unwrap();

    ctx.unauthenticated_router
        .post("/authentication/reset-password")
        .json(&json!({ "email": "test.user@example.be" }))
        .await
        .assert_status(StatusCode::OK);

    let res = ctx
        .unauthenticated_router
        .post("/authentication/reset-password")
        .json(&json!({ "email": "test.user@example.be" }))
        .await;

    res.assert_status(StatusCode::CONFLICT);
    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "RESET_PASSWORD_TOKEN_ALREADY_EXISTS");

    sqlx::query(
        "UPDATE drivers SET mail_preferences = mail_preferences & ~2 WHERE pk_driver_id = $1",
    )
    .bind(ctx.authenticated_user_id)
    .execute(&ctx.repositories.pool)
    .await
    .unwrap();
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_request_password_reset_driver_not_found(ctx: &mut context::TestContext) {
    let res = ctx
        .unauthenticated_router
        .post("/authentication/reset-password")
        .json(&json!({ "email": "unknown.driver@example.be" }))
        .await;

    res.assert_status(StatusCode::NOT_FOUND);
    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "DRIVER_NOT_FOUND");
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_request_password_reset_invalid_body(ctx: &mut context::TestContext) {
    let res = ctx
        .unauthenticated_router
        .post("/authentication/reset-password")
        .json(&json!({ "email": "not-an-email" }))
        .await;

    res.assert_status(StatusCode::BAD_REQUEST);
    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "BODY_VALIDATION");
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_confirm_password_reset_success(ctx: &mut context::TestContext) {
    let driver = ctx
        .repositories
        .driver_database_repository
        .get_driver_by_email("test.user@example.be".to_string())
        .await
        .unwrap();

    let original_driver = driver.clone();

    let (redis_key, redis_ttl) = ctx
        .repositories
        .driver_cache_repository
        .get_key_by_type(driver.pk_driver_id, DriverCacheKeyType::ResetPassword);
    ctx.repositories
        .driver_cache_repository
        .set_redis(redis_key.clone(), "valid-token".to_string(), redis_ttl)
        .await
        .unwrap();

    let res = ctx
        .unauthenticated_router
        .post("/authentication/confirm-reset-password")
        .json(&json!({
            "driver_id": driver.pk_driver_id,
            "token": "valid-token",
            "password": "newPassword123"
        }))
        .await;

    res.assert_status(StatusCode::OK);

    let token_after = ctx
        .repositories
        .driver_cache_repository
        .get_redis(redis_key)
        .await
        .unwrap();
    assert!(
        token_after.is_none(),
        "Token should be deleted from Redis after use"
    );

    let res_login = ctx
        .unauthenticated_router
        .post("/authentication/login")
        .json(&json!({
            "email": "test.user@example.be",
            "password": "newPassword123"
        }))
        .await;
    res_login.assert_status(StatusCode::OK);

    ctx.repositories
        .driver_database_repository
        .update_driver(original_driver)
        .await
        .unwrap();
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_confirm_password_reset_invalid_token(ctx: &mut context::TestContext) {
    let driver = ctx
        .repositories
        .driver_database_repository
        .get_driver_by_email("test.user@example.be".to_string())
        .await
        .unwrap();

    let (redis_key, redis_ttl) = ctx
        .repositories
        .driver_cache_repository
        .get_key_by_type(driver.pk_driver_id, DriverCacheKeyType::ResetPassword);
    ctx.repositories
        .driver_cache_repository
        .set_redis(redis_key, "correct-token".to_string(), redis_ttl)
        .await
        .unwrap();

    let res = ctx
        .unauthenticated_router
        .post("/authentication/confirm-reset-password")
        .json(&json!({
            "driver_id": driver.pk_driver_id,
            "token": "wrong-token",
            "password": "newPassword123"
        }))
        .await;

    res.assert_status(StatusCode::BAD_REQUEST);
    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "INVALID_RESET_PASSWORD_TOKEN");
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_confirm_password_reset_no_token_in_redis(ctx: &mut context::TestContext) {
    let res = ctx
        .unauthenticated_router
        .post("/authentication/confirm-reset-password")
        .json(&json!({
            "driver_id": Uuid::new_v4(),
            "token": "some-token",
            "password": "newPassword123"
        }))
        .await;

    res.assert_status(StatusCode::BAD_REQUEST);
    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "INVALID_RESET_PASSWORD_TOKEN");
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_confirm_password_reset_invalid_body(ctx: &mut context::TestContext) {
    let res = ctx
        .unauthenticated_router
        .post("/authentication/confirm-reset-password")
        .json(&json!({
            "driver_id": Uuid::new_v4(),
            "token": "some-token",
            "password": "short"
        }))
        .await;

    res.assert_status(StatusCode::BAD_REQUEST);
    let body: ErrorBody = res.json();
    assert_eq!(body.error_code, "BODY_VALIDATION");
}
