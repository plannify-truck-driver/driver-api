#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use argon2::{
        Algorithm, Argon2, Params, Version,
        password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
    };
    use bytes::Bytes;

    use crate::{
        Service, ServiceConfig,
        domain::{
            document::port::MockDocumentExternalRepository,
            driver::{
                entities::{CreateDriverRequest, DriverLanguage, DriverRow, LoginDriverRequest},
                port::{
                    DriverCacheKeyType, DriverCacheRepository, DriverDatabaseRepository,
                    DriverService, MockDriverCacheRepository, MockDriverDatabaseRepository,
                },
            },
            driver_information::port::{
                MockDriverInformationCacheRepository, MockDriverInformationDatabaseRepository,
            },
            health::port::MockHealthRepository,
            mail::port::{MailSmtpRepository, MockMailCacheRepository, MockMailDatabaseRepository},
            storage::port::MockStorageRepository,
            test::create_mock_service,
            update::port::{MockUpdateCacheRepository, MockUpdateDatabaseRepository},
            workday::port::{MockWorkdayCacheRepository, MockWorkdayDatabaseRepository},
        },
        infrastructure::{
            driver::repositories::error::DriverError, mail::repositories::error::MailError,
        },
    };

    type SuspiciousLoginCall = (String, Option<chrono::DateTime<chrono::Utc>>);

    /// Records calls to `send_driver_suspicious_login_email` so tests can assert
    /// exactly when the alert email fires, without sending anything for real.
    #[derive(Clone)]
    struct MailSmtpSpy {
        suspicious_login_calls: Arc<Mutex<Vec<SuspiciousLoginCall>>>,
    }

    impl MailSmtpSpy {
        fn new() -> Self {
            Self {
                suspicious_login_calls: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn suspicious_login_call_count(&self) -> usize {
            self.suspicious_login_calls.lock().unwrap().len()
        }

        fn last_suspicious_login_unlock_at(&self) -> Option<chrono::DateTime<chrono::Utc>> {
            self.suspicious_login_calls.lock().unwrap().last()?.1
        }
    }

    impl MailSmtpRepository for MailSmtpSpy {
        fn send_email(
            &self,
            _to: String,
            _subject: String,
            _body: String,
        ) -> Result<(), MailError> {
            Ok(())
        }

        async fn send_driver_creation_email(
            &self,
            _driver: DriverRow,
            _verify_value: String,
            _verify_ttl: u64,
        ) -> Result<(), MailError> {
            Ok(())
        }

        async fn send_driver_verification_email(
            &self,
            _driver: DriverRow,
            _verify_value: String,
            _verify_ttl: u64,
        ) -> Result<(), MailError> {
            Ok(())
        }

        async fn send_driver_deactivation_email(
            &self,
            _driver: DriverRow,
        ) -> Result<(), MailError> {
            Ok(())
        }

        async fn send_driver_reactivation_email(
            &self,
            _driver: DriverRow,
        ) -> Result<(), MailError> {
            Ok(())
        }

        async fn send_driver_reset_password_email(
            &self,
            _driver: DriverRow,
            _reset_value: String,
            _reset_ttl: u64,
        ) -> Result<(), MailError> {
            Ok(())
        }

        async fn send_driver_email_change_email(
            &self,
            _driver: DriverRow,
        ) -> Result<(), MailError> {
            Ok(())
        }

        async fn send_driver_password_change_email(
            &self,
            _driver: DriverRow,
        ) -> Result<(), MailError> {
            Ok(())
        }

        async fn send_driver_suspicious_login_email(
            &self,
            driver: DriverRow,
            unlock_at: Option<chrono::DateTime<chrono::Utc>>,
        ) -> Result<(), MailError> {
            self.suspicious_login_calls
                .lock()
                .unwrap()
                .push((driver.email, unlock_at));
            Ok(())
        }

        async fn send_driver_monthly_report_email(
            &self,
            _driver: DriverRow,
            _month: u32,
            _year: i32,
            _pdf_bytes: Bytes,
            _file_name: String,
        ) -> Result<(), MailError> {
            Ok(())
        }
    }

    type DriverTestService = Service<
        MockHealthRepository,
        MockDriverDatabaseRepository,
        MockDriverCacheRepository,
        MockWorkdayDatabaseRepository,
        MockWorkdayCacheRepository,
        MailSmtpSpy,
        MockMailDatabaseRepository,
        MockMailCacheRepository,
        MockUpdateDatabaseRepository,
        MockUpdateCacheRepository,
        MockDocumentExternalRepository,
        MockStorageRepository,
        MockDriverInformationDatabaseRepository,
        MockDriverInformationCacheRepository,
    >;

    fn make_service(mail_smtp_repository: MailSmtpSpy) -> DriverTestService {
        Service::new(
            MockHealthRepository,
            MockDriverDatabaseRepository::new(),
            MockDriverCacheRepository::new(),
            MockWorkdayDatabaseRepository::new(),
            MockWorkdayCacheRepository::new(),
            mail_smtp_repository,
            MockMailDatabaseRepository::new(),
            MockMailCacheRepository::new(),
            MockUpdateDatabaseRepository::new(),
            MockUpdateCacheRepository::new(),
            MockDocumentExternalRepository,
            MockStorageRepository::new(),
            MockDriverInformationDatabaseRepository::new(),
            MockDriverInformationCacheRepository::new(),
            ServiceConfig::default(),
        )
    }

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
        driver_database_repository: &MockDriverDatabaseRepository,
        email: &str,
        password: &str,
    ) -> DriverRow {
        driver_database_repository
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
        let driver = create_test_driver(
            &service.driver_database_repository,
            "driver@example.com",
            "correct-password",
        )
        .await;

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
        let driver = create_test_driver(
            &service.driver_database_repository,
            "driver@example.com",
            "correct-password",
        )
        .await;

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
        let driver = create_test_driver(
            &service.driver_database_repository,
            "driver@example.com",
            "correct-password",
        )
        .await;

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

    #[tokio::test]
    async fn test_login_sends_suspicious_login_email_when_attempts_exhausted()
    -> Result<(), Box<dyn std::error::Error>> {
        let spy = MailSmtpSpy::new();
        let service = make_service(spy.clone());
        let driver = create_test_driver(
            &service.driver_database_repository,
            "driver@example.com",
            "correct-password",
        )
        .await;

        let (key, ttl) = service.driver_cache_repository.get_key_by_type(
            driver.pk_driver_id,
            DriverCacheKeyType::LoginAttemptsLimitation,
        );
        // Only one attempt left: this next failure crosses the threshold.
        service
            .driver_cache_repository
            .set_redis(key, "1".to_string(), ttl)
            .await?;

        service
            .login_driver(LoginDriverRequest {
                email: "driver@example.com".to_string(),
                password: "wrong-password".to_string(),
            })
            .await
            .expect_err("login_driver should have returned an error");

        // The alert is sent on a spawned task; let it run before asserting.
        tokio::task::yield_now().await;

        assert_eq!(
            spy.suspicious_login_call_count(),
            1,
            "the alert email must be sent exactly on the attempt that exhausts the budget"
        );

        let unlock_at = spy
            .last_suspicious_login_unlock_at()
            .expect("the email should carry the lockout's unlock time");
        let seconds_until_unlock = (unlock_at - chrono::Utc::now()).num_seconds();
        assert!(
            (0..=ttl as i64).contains(&seconds_until_unlock),
            "unlock_at should fall within the lockout window, got {} seconds",
            seconds_until_unlock
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_login_does_not_resend_suspicious_login_email_once_locked()
    -> Result<(), Box<dyn std::error::Error>> {
        let spy = MailSmtpSpy::new();
        let service = make_service(spy.clone());
        let driver = create_test_driver(
            &service.driver_database_repository,
            "driver@example.com",
            "correct-password",
        )
        .await;

        let (key, ttl) = service.driver_cache_repository.get_key_by_type(
            driver.pk_driver_id,
            DriverCacheKeyType::LoginAttemptsLimitation,
        );
        // Already locked out from a previous attempt.
        service
            .driver_cache_repository
            .set_redis(key, "0".to_string(), ttl)
            .await?;

        service
            .login_driver(LoginDriverRequest {
                email: "driver@example.com".to_string(),
                password: "correct-password".to_string(),
            })
            .await
            .expect_err("login_driver should reject a locked-out account");

        tokio::task::yield_now().await;

        assert_eq!(
            spy.suspicious_login_call_count(),
            0,
            "no new alert should be sent while the account is already locked"
        );

        Ok(())
    }
}
