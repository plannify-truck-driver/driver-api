use crate::{
    Service,
    domain::{
        document::port::DocumentExternalRepository,
        driver::{
            entities::{
                CreateDriverRequest, CreateDriverRestPeriodRequest, DriverRestPeriod, DriverRow,
                LoginDriverRequest, UpdateDriverRequest,
            },
            port::{
                DriverCacheKeyType, DriverCacheRepository, DriverDatabaseRepository, DriverService,
                to_email_case, to_title_case,
            },
        },
        health::port::HealthRepository,
        mail::port::{MailCacheRepository, MailDatabaseRepository, MailSmtpRepository},
        storage::port::StorageRepository,
        update::port::{UpdateCacheRepository, UpdateDatabaseRepository},
        workday::port::{WorkdayCacheRepository, WorkdayDatabaseRepository},
    },
    infrastructure::driver::repositories::error::DriverError,
};
use argon2::{
    Algorithm, Argon2, Params, Version,
    password_hash::PasswordHash,
    password_hash::{PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use tracing::error;
use uuid::Uuid;

impl<H, DD, DC, WD, WC, MS, MD, MC, UD, UC, DE, DS> DriverService
    for Service<H, DD, DC, WD, WC, MS, MD, MC, UD, UC, DE, DS>
where
    H: HealthRepository,
    DD: DriverDatabaseRepository,
    DC: DriverCacheRepository,
    WD: WorkdayDatabaseRepository,
    WC: WorkdayCacheRepository,
    MS: MailSmtpRepository,
    MD: MailDatabaseRepository,
    MC: MailCacheRepository,
    UD: UpdateDatabaseRepository,
    UC: UpdateCacheRepository,
    DE: DocumentExternalRepository,
    DS: StorageRepository,
{
    #[tracing::instrument(
        name = "driver_service.create_driver",
        skip(self),
        fields(
            firstname = %create_request.firstname,
            lastname = %create_request.lastname,
            email = %create_request.email,
            limitation_hit = tracing::field::Empty,
            email_in_denylist = tracing::field::Empty,
        )
    )]
    async fn create_driver(
        &self,
        mut create_request: CreateDriverRequest,
        email_list_deny: Vec<String>,
    ) -> Result<DriverRow, DriverError> {
        let limitation = self
            .driver_database_repository
            .get_actual_driver_limitation()
            .await?;

        tracing::Span::current().record("limitation_hit", limitation.is_some());

        if let Some(limitation_info) = limitation {
            let current_drivers = self
                .driver_database_repository
                .get_number_of_drivers()
                .await?;
            if current_drivers >= limitation_info.maximum_limit as i64 {
                return Err(DriverError::DriverLimitReached {
                    start_at: limitation_info.start_at.to_rfc3339(),
                    end_at: limitation_info.end_at.map(|dt| dt.to_rfc3339()),
                });
            }
        }

        create_request.firstname = to_title_case(create_request.firstname);
        create_request.lastname = to_title_case(create_request.lastname);
        create_request.email = to_email_case(create_request.email);

        let email_domain = create_request.email.split('@').next_back().unwrap_or("");

        if email_list_deny.contains(&email_domain.to_string()) {
            error!(
                "Attempt to sign up with denylisted email domain ({}) : {}",
                email_domain, create_request.email
            );
            tracing::Span::current().record("email_in_denylist", true);

            return Err(DriverError::EmailDomainDenylisted {
                domain: email_domain.to_string(),
            });
        }

        tracing::Span::current().record("email_in_denylist", false);

        let salt = SaltString::generate(&mut OsRng);
        let params = Params::new(19 * 1024, 2, 1, None).map_err(|e| {
            error!("Failed to create Argon2 params for password hashing: {}", e);
            DriverError::Internal
        })?; // 19 MiB, 2 itérations, 1 thread
        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

        let password_hash = argon2
            .hash_password(create_request.password.as_bytes(), &salt)
            .map_err(|e| {
                error!("Failed to hash password: {}", e);
                DriverError::Internal
            })?;
        create_request.password = password_hash.to_string();

        let driver = self
            .driver_database_repository
            .create_driver(create_request)
            .await?;

        Ok(driver)
    }

    #[tracing::instrument(
        name = "driver_service.login_driver",
        skip(self),
        fields(
            email = %login_request.email,
        )
    )]
    async fn login_driver(
        &self,
        login_request: LoginDriverRequest,
    ) -> Result<DriverRow, DriverError> {
        let email = to_email_case(login_request.email);
        let driver = self
            .driver_database_repository
            .get_driver_by_email(email)
            .await
            .map_err(|_| DriverError::InvalidCredentials)?;

        let params = Params::new(19 * 1024, 2, 1, None).map_err(|e| {
            error!(
                "Failed to create Argon2 params for refresh token hashing: {}",
                e
            );
            DriverError::Internal
        })?; // 19 MiB, 2 itérations, 1 thread
        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

        let parsed_hash = PasswordHash::new(&driver.password_hash).map_err(|e| {
            error!("Failed to parse password hash: {}", e);
            DriverError::Internal
        })?;
        match argon2
            .verify_password(login_request.password.as_bytes(), &parsed_hash)
            .is_ok()
        {
            true => (),
            false => return Err(DriverError::InvalidCredentials),
        }

        let suspension = self
            .driver_database_repository
            .get_current_driver_suspension(driver.pk_driver_id)
            .await?;

        if let Some(suspension_info) = suspension
            && !suspension_info.can_access_restricted_space
        {
            return Err(DriverError::DriverSuspension {
                message: suspension_info.driver_message,
                start_at: suspension_info.start_at.to_rfc3339(),
                end_at: suspension_info.end_at.map(|dt| dt.to_rfc3339()),
            });
        }

        Ok(driver)
    }

    #[tracing::instrument(
        name = "driver_service.get_driver_by_id",
        skip(self),
        fields(
            driver_id = %driver_id,
        )
    )]
    async fn get_driver_by_id(&self, driver_id: Uuid) -> Result<Option<DriverRow>, DriverError> {
        let driver = self
            .driver_database_repository
            .get_driver_by_id(driver_id)
            .await?;

        Ok(driver)
    }

    #[tracing::instrument(
        name = "driver_service.verify_driver_account",
        skip(self),
        fields(
            driver_id = %driver_id,
        )
    )]
    async fn verify_driver_account(
        &self,
        driver_id: Uuid,
        token: String,
    ) -> Result<DriverRow, DriverError> {
        let driver = self
            .driver_database_repository
            .get_driver_by_id(driver_id)
            .await
            .map_err(|_| DriverError::InvalidVerificationKey)?;

        let mut driver = driver.ok_or(DriverError::InvalidVerificationKey)?;

        let (redis_key, _) = self
            .driver_cache_repository
            .get_key_by_type(driver.pk_driver_id, DriverCacheKeyType::VerifyEmail);
        let verify_value = self.driver_cache_repository.get_redis(redis_key).await?;

        if verify_value != Some(token) {
            return Err(DriverError::InvalidVerificationKey);
        }

        if driver.verified_at.is_some() {
            return Err(DriverError::AccountAlreadyVerified);
        }

        driver.verified_at = Some(chrono::Utc::now());

        self.driver_database_repository
            .update_driver(driver.clone())
            .await?;

        Ok(driver)
    }

    async fn get_driver_for_refresh(&self, driver_id: Uuid) -> Result<DriverRow, DriverError> {
        let driver = self
            .driver_database_repository
            .get_driver_by_id(driver_id)
            .await?
            .ok_or(DriverError::InvalidRefreshToken)?;

        Ok(driver)
    }

    async fn generate_tokens<F>(
        &self,
        mut driver: DriverRow,
        create_tokens: F,
        access_ttl: u64,
        refresh_ttl: u64,
        domain_name: &str,
    ) -> Result<(String, String, String), DriverError>
    where
        F: Fn(&DriverRow) -> Result<(String, String), DriverError> + Send + Sync,
    {
        let (access_token, refresh_token) = create_tokens(&driver)?;

        let domain_host = domain_name
            .trim_start_matches("http://")
            .trim_start_matches("https://")
            .split(":")
            .next()
            .unwrap_or(domain_name);
        let domain_parts: Vec<&str> = domain_host.split('.').collect();
        let domain = if domain_parts.len() >= 2 {
            format!(
                ".{}.{}",
                domain_parts[domain_parts.len() - 2],
                domain_parts[domain_parts.len() - 1]
            )
        } else {
            domain_host.to_string()
        };

        let access_token_cookie = format!(
            "access_token={}; Path=/; Domain={}; HttpOnly; Secure; SameSite=Lax; Max-Age={}",
            access_token, domain, access_ttl
        );

        let refresh_token_cookie = format!(
            "refresh_token={}; Path=/; Domain={}; HttpOnly; Secure; SameSite=Lax; Max-Age={}",
            refresh_token, domain, refresh_ttl
        );

        driver.last_login_at = Some(chrono::Utc::now());
        self.driver_database_repository
            .update_driver(driver)
            .await?;

        Ok((access_token, access_token_cookie, refresh_token_cookie))
    }

    #[tracing::instrument(name = "driver_service.delete_refresh_token", skip(self))]
    async fn delete_refresh_token(&self, domain_name: &str) -> Result<String, DriverError> {
        let domain_host = domain_name
            .trim_start_matches("http://")
            .trim_start_matches("https://")
            .split(":")
            .next()
            .unwrap_or(domain_name);
        let domain_parts: Vec<&str> = domain_host.split('.').collect();
        let mut domain = if domain_parts.len() >= 2 {
            format!(
                "{}.{}",
                domain_parts[domain_parts.len() - 2],
                domain_parts[domain_parts.len() - 1]
            )
        } else {
            domain_host.to_string()
        };

        if domain != "localhost" {
            domain = format!(".{}", domain);
        }

        let refresh_token_cookie = format!(
            "refresh_token=''; Path=/; Domain={}; HttpOnly; Secure; SameSite=Lax; Max-Age={}",
            domain, 0
        );

        Ok(refresh_token_cookie)
    }

    #[tracing::instrument(
        name = "driver_service.get_driver_rest_periods",
        skip(self),
        fields(
            driver_id = %driver_id,
        )
    )]
    async fn get_driver_rest_periods(
        &self,
        driver_id: Uuid,
    ) -> Result<Vec<DriverRestPeriod>, DriverError> {
        let rest_periods = self
            .driver_database_repository
            .get_driver_rest_periods(driver_id)
            .await?;
        Ok(rest_periods)
    }

    #[tracing::instrument(
        name = "driver_service.set_driver_rest_periods",
        skip(self),
        fields(
            driver_id = %driver_id,
        )
    )]
    async fn set_driver_rest_periods(
        &self,
        driver_id: Uuid,
        rest_periods: Vec<CreateDriverRestPeriodRequest>,
    ) -> Result<(), DriverError> {
        let mut rest_periods_service: Vec<DriverRestPeriod> =
            Vec::from_iter(rest_periods.into_iter().map(|period| DriverRestPeriod {
                start: period.start,
                end: period.end,
                rest: period.rest,
            }));
        rest_periods_service.sort_by_key(|period| period.start);

        for index in 0..rest_periods_service.len() {
            if index == 0 {
                if rest_periods_service[index].start != "00:00:00".parse().unwrap() {
                    return Err(DriverError::InvalidRestPeriod {
                        details: format!(
                            "The first rest period must start at 00:00:00, got {}",
                            rest_periods_service[index].start
                        ),
                    });
                }
            } else {
                let expected_start =
                    rest_periods_service[index - 1].end + chrono::Duration::seconds(1);
                if rest_periods_service[index].start != expected_start {
                    return Err(DriverError::InvalidRestPeriod {
                        details: format!(
                            "Rest period at index {} starts at {} but previous period ends at {}, need one second gap. The correct start time must be {}.",
                            index,
                            rest_periods_service[index].start,
                            rest_periods_service[index - 1].end,
                            expected_start
                        ),
                    });
                }
            }

            if index == rest_periods_service.len() - 1
                && rest_periods_service[index].end != "23:59:59".parse().unwrap()
            {
                return Err(DriverError::InvalidRestPeriod {
                    details: format!(
                        "The last rest period must end at 23:59:59, got {}",
                        rest_periods_service[index].end
                    ),
                });
            }

            if rest_periods_service[index].start >= rest_periods_service[index].end {
                return Err(DriverError::InvalidRestPeriod {
                    details: format!(
                        "Rest period at index {} has start time {} which is not before end time {}.",
                        index, rest_periods_service[index].start, rest_periods_service[index].end
                    ),
                });
            }
        }

        self.driver_database_repository
            .set_driver_rest_periods(driver_id, rest_periods_service)
            .await
    }

    #[tracing::instrument(
        name = "driver_service.delete_driver_rest_periods",
        skip(self),
        fields(
            driver_id = %driver_id,
        )
    )]
    async fn delete_driver_rest_periods(&self, driver_id: Uuid) -> Result<(), DriverError> {
        self.driver_database_repository
            .delete_driver_rest_periods(driver_id)
            .await
    }

    #[tracing::instrument(
        name = "driver_service.update_driver_info",
        skip(self),
        fields(
            driver_id = %driver_id,
            email_changed = tracing::field::Empty,
            email_in_denylist = tracing::field::Empty,
        )
    )]
    async fn update_driver_info(
        &self,
        driver_id: Uuid,
        update_request: UpdateDriverRequest,
        email_list_deny: Vec<String>,
    ) -> Result<(DriverRow, bool), DriverError> {
        let mut driver = self
            .driver_database_repository
            .get_driver_by_id(driver_id)
            .await?
            .ok_or(DriverError::DriverNotFound)?;

        let mut email_changed = false;

        if let Some(firstname) = update_request.firstname {
            driver.firstname = to_title_case(firstname);
        }

        if let Some(lastname) = update_request.lastname {
            driver.lastname = to_title_case(lastname);
        }

        match update_request.gender {
            Some(Some(gender)) => driver.gender = Some(gender.to_uppercase()),
            Some(None) => driver.gender = None,
            None => {}
        }

        if let Some(email) = update_request.email {
            let new_email = to_email_case(email);
            if new_email != driver.email {
                let email_domain = new_email.split('@').next_back().unwrap_or("");
                if email_list_deny.contains(&email_domain.to_string()) {
                    error!(
                        "Attempt to update email to a denylisted domain ({}) for driver {}",
                        email_domain, driver_id
                    );
                    tracing::Span::current().record("email_in_denylist", true);
                    return Err(DriverError::EmailDomainDenylisted {
                        domain: email_domain.to_string(),
                    });
                }
                tracing::Span::current().record("email_in_denylist", false);
                driver.email = new_email;
                driver.verified_at = None;
                email_changed = true;
            }
        }

        if let Some(password) = update_request.password {
            let salt = SaltString::generate(&mut OsRng);
            let params = Params::new(19 * 1024, 2, 1, None).map_err(|e| {
                error!("Failed to create Argon2 params for password hashing: {}", e);
                DriverError::Internal
            })?;
            let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
            let password_hash = argon2
                .hash_password(password.as_bytes(), &salt)
                .map_err(|e| {
                    error!("Failed to hash password: {}", e);
                    DriverError::Internal
                })?;
            driver.password_hash = password_hash.to_string();
        }

        match update_request.phone_number {
            Some(Some(phone_number)) => driver.phone_number = Some(phone_number),
            Some(None) => driver.phone_number = None,
            None => {}
        }

        if let Some(language) = update_request.language {
            driver.language = language.to_string();
        }

        tracing::Span::current().record("email_changed", email_changed);

        let updated_driver = self
            .driver_database_repository
            .update_driver(driver)
            .await?;

        Ok((updated_driver, email_changed))
    }

    #[tracing::instrument(
        name = "driver_service.deactivate_driver",
        skip(self),
        fields(driver_id = %driver_id)
    )]
    async fn deactivate_driver(&self, driver_id: Uuid) -> Result<DriverRow, DriverError> {
        let mut driver = self
            .driver_database_repository
            .get_driver_by_id(driver_id)
            .await?
            .ok_or(DriverError::DriverNotFound)?;

        if driver.deactivated_at.is_some() {
            return Err(DriverError::AccountAlreadyDeactivated);
        }

        driver.deactivated_at = Some(
            chrono::Utc::now() + chrono::Duration::days(self.config.account_deactivation_days),
        );

        let updated_driver = self
            .driver_database_repository
            .update_driver(driver)
            .await?;

        Ok(updated_driver)
    }

    #[tracing::instrument(
        name = "driver_service.reactivate_driver",
        skip(self),
        fields(driver_id = %driver_id)
    )]
    async fn reactivate_driver(&self, driver_id: Uuid) -> Result<DriverRow, DriverError> {
        let mut driver = self
            .driver_database_repository
            .get_driver_by_id(driver_id)
            .await?
            .ok_or(DriverError::DriverNotFound)?;

        if driver.deactivated_at.is_none() {
            return Err(DriverError::AccountNotDeactivated);
        }

        driver.deactivated_at = None;

        let updated_driver = self
            .driver_database_repository
            .update_driver(driver)
            .await?;

        Ok(updated_driver)
    }

    #[tracing::instrument(
        name = "driver_service.request_password_reset",
        skip(self),
        fields(email = %email)
    )]
    async fn request_password_reset(&self, email: String) -> Result<DriverRow, DriverError> {
        let email = to_email_case(email);
        let driver = self
            .driver_database_repository
            .get_driver_by_email(email)
            .await?;

        let (redis_key, _) = self
            .driver_cache_repository
            .get_key_by_type(driver.pk_driver_id, DriverCacheKeyType::ResetPassword);
        let existing_token = self.driver_cache_repository.get_redis(redis_key).await?;

        if existing_token.is_some() {
            return Err(DriverError::ResetPasswordTokenAlreadyExists);
        }

        Ok(driver)
    }

    #[tracing::instrument(
        name = "driver_service.confirm_password_reset",
        skip(self),
        fields(driver_id = %driver_id)
    )]
    async fn confirm_password_reset(
        &self,
        driver_id: Uuid,
        token: String,
        new_password: String,
    ) -> Result<(), DriverError> {
        let (redis_key, _) = self
            .driver_cache_repository
            .get_key_by_type(driver_id, DriverCacheKeyType::ResetPassword);
        let stored_token = self
            .driver_cache_repository
            .get_redis(redis_key.clone())
            .await?;

        if stored_token != Some(token) {
            return Err(DriverError::InvalidResetPasswordToken);
        }

        let mut driver = self
            .driver_database_repository
            .get_driver_by_id(driver_id)
            .await?
            .ok_or(DriverError::InvalidResetPasswordToken)?;

        let salt = SaltString::generate(&mut OsRng);
        let params = Params::new(19 * 1024, 2, 1, None).map_err(|e| {
            error!("Failed to create Argon2 params for password hashing: {}", e);
            DriverError::Internal
        })?;
        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
        let password_hash = argon2
            .hash_password(new_password.as_bytes(), &salt)
            .map_err(|e| {
                error!("Failed to hash password: {}", e);
                DriverError::Internal
            })?;
        driver.password_hash = password_hash.to_string();

        self.driver_database_repository
            .update_driver(driver)
            .await?;

        self.driver_cache_repository.delete_redis(redis_key).await?;

        Ok(())
    }
}
