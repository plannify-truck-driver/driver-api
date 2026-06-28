use axum::http::StatusCode;
use serde_json::json;
use serial_test::serial;
use test_context::test_context;

use crate::context;

const BASELINE_EMAIL: &str = "test.user@example.be";
const BASELINE_PASSWORD_HASH: &str =
    "$argon2id$v=19$m=19456,t=2,p=1$GvJ0zPtHLrLN0ubKYXtqdw$dAqS9mMzUO55YVmiWPESW60AagJ5px+803z3nuEmH48";
/// AccountChangement mail type (id=3): bit = 1 << (3 - 1) = 4
const ACCOUNT_CHANGEMENT_BIT: i32 = 4;

/// Reset the test driver to its test-dataset values.
///
/// This must be called at the start AND end of every test in this module to ensure
/// that a previous crashed run cannot leave dirty state (email, mail_preferences,
/// password_hash, verified_at) that would break other tests — notably
/// `get_mail_preferences::test_get_mail_preferences_success`, which asserts
/// `mail_preferences = 0` and runs before this module alphabetically (now fixed by
/// putting `mail_preference_guard` first in mod.rs).
///
/// Also removes any phantom driver that may have stolen the baseline email when
/// the test driver's email was changed away and a signup test ran.
async fn reset_driver_to_baseline(ctx: &mut context::TestContext) {
    sqlx::query("DELETE FROM drivers WHERE email = $1 AND pk_driver_id != $2")
        .bind(BASELINE_EMAIL)
        .bind(ctx.authenticated_user_id)
        .execute(&ctx.repositories.pool)
        .await
        .unwrap();

    sqlx::query(
        "UPDATE drivers
         SET email           = $1,
             mail_preferences = 0,
             password_hash   = $2,
             verified_at     = '2026-01-01 00:00:00'
         WHERE pk_driver_id = $3",
    )
    .bind(BASELINE_EMAIL)
    .bind(BASELINE_PASSWORD_HASH)
    .bind(ctx.authenticated_user_id)
    .execute(&ctx.repositories.pool)
    .await
    .unwrap();
}

async fn enable_account_changement(ctx: &mut context::TestContext) {
    sqlx::query(
        "UPDATE drivers SET mail_preferences = mail_preferences | $1 WHERE pk_driver_id = $2",
    )
    .bind(ACCOUNT_CHANGEMENT_BIT)
    .bind(ctx.authenticated_user_id)
    .execute(&ctx.repositories.pool)
    .await
    .unwrap();
}

async fn disable_account_changement(ctx: &mut context::TestContext) {
    sqlx::query(
        "UPDATE drivers SET mail_preferences = mail_preferences & ~$1 WHERE pk_driver_id = $2",
    )
    .bind(ACCOUNT_CHANGEMENT_BIT)
    .bind(ctx.authenticated_user_id)
    .execute(&ctx.repositories.pool)
    .await
    .unwrap();
}

async fn count_mails_by_description(ctx: &context::TestContext, description: &str) -> i64 {
    sqlx::query_scalar(
        "SELECT COUNT(*) FROM driver_mails WHERE fk_driver_id = $1 AND description = $2",
    )
    .bind(ctx.authenticated_user_id)
    .bind(description)
    .fetch_one(&ctx.repositories.pool)
    .await
    .unwrap()
}

// ── Email change notification ─────────────────────────────────────────────────

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_email_change_notification_skipped_when_preference_disabled(
    ctx: &mut context::TestContext,
) {
    reset_driver_to_baseline(ctx).await;
    disable_account_changement(ctx).await;

    let before = count_mails_by_description(ctx, "Driver email change notification").await;

    let res = ctx
        .authenticated_router
        .patch("/me")
        .json(&json!({ "email": "changed@test.be" }))
        .await;

    res.assert_status(StatusCode::OK);

    let after = count_mails_by_description(ctx, "Driver email change notification").await;
    assert_eq!(
        before, after,
        "no email change notification should be created when AccountChangement preference is disabled"
    );

    reset_driver_to_baseline(ctx).await;
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_email_change_notification_created_when_preference_enabled(
    ctx: &mut context::TestContext,
) {
    reset_driver_to_baseline(ctx).await;
    enable_account_changement(ctx).await;

    let before = count_mails_by_description(ctx, "Driver email change notification").await;

    let res = ctx
        .authenticated_router
        .patch("/me")
        .json(&json!({ "email": "changed@test.be" }))
        .await;

    res.assert_status(StatusCode::OK);

    let after = count_mails_by_description(ctx, "Driver email change notification").await;
    assert_eq!(
        after,
        before + 1,
        "one email change notification should be created when AccountChangement preference is enabled"
    );

    reset_driver_to_baseline(ctx).await;
}

// ── Password change notification (via profile update) ─────────────────────────

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_password_change_notification_skipped_when_preference_disabled(
    ctx: &mut context::TestContext,
) {
    reset_driver_to_baseline(ctx).await;
    disable_account_changement(ctx).await;

    let before = count_mails_by_description(ctx, "Driver password change notification").await;

    let res = ctx
        .authenticated_router
        .patch("/me")
        .json(&json!({ "password": "newPassword456!" }))
        .await;

    res.assert_status(StatusCode::OK);

    let after = count_mails_by_description(ctx, "Driver password change notification").await;
    assert_eq!(
        before, after,
        "no password change notification should be created when AccountChangement preference is disabled"
    );

    reset_driver_to_baseline(ctx).await;
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_password_change_notification_created_when_preference_enabled(
    ctx: &mut context::TestContext,
) {
    reset_driver_to_baseline(ctx).await;
    enable_account_changement(ctx).await;

    let before = count_mails_by_description(ctx, "Driver password change notification").await;

    let res = ctx
        .authenticated_router
        .patch("/me")
        .json(&json!({ "password": "newPassword456!" }))
        .await;

    res.assert_status(StatusCode::OK);

    let after = count_mails_by_description(ctx, "Driver password change notification").await;
    assert_eq!(
        after,
        before + 1,
        "one password change notification should be created when AccountChangement preference is enabled"
    );

    reset_driver_to_baseline(ctx).await;
}

// ── Password change notification (via reset) ──────────────────────────────────

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_confirm_reset_password_notification_skipped_when_preference_disabled(
    ctx: &mut context::TestContext,
) {
    use plannify_driver_api_core::domain::driver::port::{DriverCacheKeyType, DriverCacheRepository};

    reset_driver_to_baseline(ctx).await;
    disable_account_changement(ctx).await;

    let (redis_key, redis_ttl) = ctx
        .repositories
        .driver_cache_repository
        .get_key_by_type(ctx.authenticated_user_id, DriverCacheKeyType::ResetPassword);
    ctx.repositories
        .driver_cache_repository
        .set_redis(redis_key, "reset-token".to_string(), redis_ttl)
        .await
        .unwrap();

    let before = count_mails_by_description(ctx, "Driver password change notification").await;

    let res = ctx
        .unauthenticated_router
        .post("/authentication/confirm-reset-password")
        .json(&json!({
            "driver_id": ctx.authenticated_user_id,
            "token": "reset-token",
            "password": "newPassword456!"
        }))
        .await;

    res.assert_status(StatusCode::OK);

    let after = count_mails_by_description(ctx, "Driver password change notification").await;
    assert_eq!(
        before, after,
        "no password change notification should be created when AccountChangement preference is disabled"
    );

    reset_driver_to_baseline(ctx).await;
}

#[test_context(context::TestContext)]
#[tokio::test]
#[serial]
async fn test_confirm_reset_password_notification_created_when_preference_enabled(
    ctx: &mut context::TestContext,
) {
    use plannify_driver_api_core::domain::driver::port::{DriverCacheKeyType, DriverCacheRepository};

    reset_driver_to_baseline(ctx).await;
    enable_account_changement(ctx).await;

    let (redis_key, redis_ttl) = ctx
        .repositories
        .driver_cache_repository
        .get_key_by_type(ctx.authenticated_user_id, DriverCacheKeyType::ResetPassword);
    ctx.repositories
        .driver_cache_repository
        .set_redis(redis_key, "reset-token".to_string(), redis_ttl)
        .await
        .unwrap();

    let before = count_mails_by_description(ctx, "Driver password change notification").await;

    let res = ctx
        .unauthenticated_router
        .post("/authentication/confirm-reset-password")
        .json(&json!({
            "driver_id": ctx.authenticated_user_id,
            "token": "reset-token",
            "password": "newPassword456!"
        }))
        .await;

    res.assert_status(StatusCode::OK);

    let after = count_mails_by_description(ctx, "Driver password change notification").await;
    assert_eq!(
        after,
        before + 1,
        "one password change notification should be created when AccountChangement preference is enabled"
    );

    reset_driver_to_baseline(ctx).await;
}
