use crate::{
    Service,
    domain::{
        driver::{
            entities::{
                CreateDriverRequest, CreateDriverRestPeriodRequest, DriverRestPeriod, DriverRow,
                LoginDriverRequest,
            },
            port::{
                DriverCacheKeyType, DriverCacheRepository, DriverDatabaseRepository, DriverService,
            },
        },
        health::port::HealthRepository,
        mail::port::{MailDatabaseRepository, MailSmtpRepository},
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

impl<H, DD, DC, WD, WC, MS, MD, UD, UC, DE> DriverService
    for Service<H, DD, DC, WD, WC, MS, MD, UD, UC, DE>
where
    H: HealthRepository,
    DD: DriverDatabaseRepository,
    DC: DriverCacheRepository,
    WD: WorkdayDatabaseRepository,
    WC: WorkdayCacheRepository,
    MS: MailSmtpRepository,
    MD: MailDatabaseRepository,
    UD: UpdateDatabaseRepository,
    UC: UpdateCacheRepository,
    DE: crate::domain::document::port::DocumentExternalRepository,
{
    fn to_title_case(name: String) -> String {
        name.trim()
            .split(|c: char| c.is_whitespace() || c == '-')
            .map(|word| {
                if word.is_empty() {
                    return String::new();
                }
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => {
                        first.to_uppercase().collect::<String>()
                            + chars.as_str().to_lowercase().as_str()
                    }
                }
            })
            .collect::<Vec<String>>()
            .join("-")
    }

    async fn create_driver(
        &self,
        mut create_request: CreateDriverRequest,
        email_list_deny: Vec<String>,
    ) -> Result<DriverRow, DriverError> {
        let limitation = self
            .driver_database_repository
            .get_actual_driver_limitation()
            .await?;
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

        create_request.firstname = Self::to_title_case(create_request.firstname);
        create_request.lastname = Self::to_title_case(create_request.lastname);
        create_request.email = create_request.email.trim().to_lowercase();

        let email_domain = create_request.email.split('@').next_back().unwrap_or("");

        if email_list_deny.contains(&email_domain.to_string()) {
            error!(
                "Attempt to sign up with denylisted email domain ({}) : {}",
                email_domain, create_request.email
            );
            return Err(DriverError::EmailDomainDenylisted {
                domain: email_domain.to_string(),
            });
        }

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

    async fn login_driver(
        &self,
        login_request: LoginDriverRequest,
    ) -> Result<DriverRow, DriverError> {
        let email = login_request.email.trim().to_lowercase();
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

    async fn get_driver_by_id(&self, driver_id: Uuid) -> Result<Option<DriverRow>, DriverError> {
        let driver = self
            .driver_database_repository
            .get_driver_by_id(driver_id)
            .await?;

        Ok(driver)
    }

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

    async fn generate_tokens<F>(
        &self,
        mut driver: DriverRow,
        create_tokens: F,
        refresh_ttl: u64,
        domain_name: &str,
    ) -> Result<(String, String), DriverError>
    where
        F: Fn(&DriverRow) -> Result<(String, String), DriverError> + Send + Sync,
    {
        let (access_token, refresh_token) = create_tokens(&driver)?;

        let domain = domain_name
            .trim_start_matches("http://")
            .trim_start_matches("https://")
            .split(":")
            .next()
            .unwrap_or(domain_name);

        let refresh_token_cookie = format!(
            "refresh_token={}; Path=/; Domain={}; HttpOnly; Secure; SameSite=Lax; Max-Age={}",
            refresh_token, domain, refresh_ttl
        );

        let salt = SaltString::generate(&mut OsRng);
        let params = Params::new(19 * 1024, 2, 1, None).map_err(|e| {
            error!(
                "Failed to create Argon2 params for refresh token hashing: {}",
                e
            );
            DriverError::Internal
        })?; // 19 MiB, 2 itérations, 1 thread
        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

        let token_hash = argon2
            .hash_password(refresh_token.as_bytes(), &salt)
            .map_err(|e| {
                error!("Failed to hash refresh token: {}", e);
                DriverError::Internal
            })?;

        driver.refresh_token_hash = Some(token_hash.to_string());
        self.driver_database_repository
            .update_driver(driver)
            .await?;

        Ok((access_token, refresh_token_cookie))
    }

    async fn delete_refresh_token(
        &self,
        domain_name: &str,
    ) -> Result<String, DriverError> {
        let domain = domain_name
            .trim_start_matches("http://")
            .trim_start_matches("https://")
            .split(":")
            .next()
            .unwrap_or(domain_name);

        let refresh_token_cookie = format!(
            "refresh_token=''; Path=/; Domain={}; HttpOnly; Secure; SameSite=Lax; Max-Age={}",
            domain, 0
        );

        Ok(refresh_token_cookie)
    }

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

    async fn delete_driver_rest_periods(&self, driver_id: Uuid) -> Result<(), DriverError> {
        self.driver_database_repository
            .delete_driver_rest_periods(driver_id)
            .await
    }
}
