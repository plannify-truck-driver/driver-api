#[cfg(test)]
mod tests {
    use argon2::{
        Algorithm, Argon2, Params, Version,
        password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
    };

    use crate::{
        domain::{
            driver::{
                entities::{CreateDriverRequest, DriverLanguage, LoginDriverRequest},
                port::{
                    DriverCacheKeyType, DriverCacheRepository, DriverDatabaseRepository,
                    DriverService,
                },
            },
            test::create_mock_service,
        },
        infrastructure::driver::repositories::error::DriverError,
    };

    fn hash_password(password: &str) -> String {
        let salt = SaltString::generate(&mut OsRng);
        let params = Params::new(19 * 1024, 2, 1, None).unwrap();
        Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
            .hash_password(password.as_bytes(), &salt)
            .unwrap()
            .to_string()
    }

    /// Creates a verified driver directly in the mock database, bypassing the
    /// `create_driver` service method (which would also touch the driver
    /// limitation/denylist checks that are irrelevant here).
    async fn create_test_driver(
        service: &crate::domain::test::MockService,
        email: &str,
        password: &str,
    ) -> crate::domain::driver::entities::DriverRow {
        service
            .driver_database_repository
            .create_driver(CreateDriverRequest {
                firstname: "John".to_string(),
                lastname: "Doe".to_string(),
                gender: None,
                email: email.to_string(),
                password: hash_password(password),
                language: DriverLanguage::FR,
            })
            .await
            .expect("create_driver returned an error")
    }

    #[tokio::test]
    async fn test_login_wrong_password_decrements_attempts_counter()
    -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();
        let driver = create_test_driver(&service, "driver@example.com", "correct-password").await;

        let error = service
            .login_driver(LoginDriverRequest {
                email: "driver@example.com".to_string(),
                password: "wrong-password".to_string(),
            })
            .await
            .expect_err("login_driver should have returned an error");
        assert!(matches!(error, DriverError::InvalidCredentials));

        let (key, _ttl) = service.driver_cache_repository.get_key_by_type(
            driver.pk_driver_id,
            DriverCacheKeyType::LoginAttemptsLimitation,
        );
        let remaining: i64 = service
            .driver_cache_repository
            .get_redis(key)
            .await?
            .expect("the counter must be initialized after the first failed attempt")
            .parse()?;

        assert_eq!(
            remaining,
            service.config.max_login_attempts - 1,
            "the counter must be decremented by exactly one after a failed attempt"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_login_fails_with_correct_password_once_attempts_exhausted()
    -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();
        let driver = create_test_driver(&service, "driver@example.com", "correct-password").await;

        let (key, ttl) = service.driver_cache_repository.get_key_by_type(
            driver.pk_driver_id,
            DriverCacheKeyType::LoginAttemptsLimitation,
        );
        service
            .driver_cache_repository
            .set_redis(key, "0".to_string(), ttl)
            .await?;

        let error = service
            .login_driver(LoginDriverRequest {
                email: "driver@example.com".to_string(),
                password: "correct-password".to_string(),
            })
            .await
            .expect_err("login_driver should reject a locked-out account");

        assert!(
            matches!(error, DriverError::InvalidCredentials),
            "a locked-out account must return the exact same error as a wrong password, \
             never a distinct 'too many attempts' signal"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_login_success_resets_attempts_counter() -> Result<(), Box<dyn std::error::Error>>
    {
        let service = create_mock_service();
        let driver = create_test_driver(&service, "driver@example.com", "correct-password").await;

        let (key, ttl) = service.driver_cache_repository.get_key_by_type(
            driver.pk_driver_id,
            DriverCacheKeyType::LoginAttemptsLimitation,
        );
        service
            .driver_cache_repository
            .set_redis(key.clone(), "3".to_string(), ttl)
            .await?;

        service
            .login_driver(LoginDriverRequest {
                email: "driver@example.com".to_string(),
                password: "correct-password".to_string(),
            })
            .await
            .expect("login_driver should have succeeded");

        let remaining = service.driver_cache_repository.get_redis(key).await?;
        assert_eq!(
            remaining, None,
            "a successful login must reset the attempts counter"
        );

        Ok(())
    }
}
