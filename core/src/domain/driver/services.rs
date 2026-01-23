use crate::{
    Service,
    domain::{
        driver::{
            entities::{
                CreateDriverRequest, CreateDriverRestPeriodRequest, DriverRestPeriod, DriverRow,
                LoginDriverRequest,
            },
            port::{DriverCacheKeyType, DriverCacheRepository, DriverRepository, DriverService},
        },
        health::port::HealthRepository,
        mail::port::MailRepository,
        workday::port::WorkdayRepository,
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

impl<H, D, DC, W, M> DriverService for Service<H, D, DC, W, M>
where
    H: HealthRepository,
    D: DriverRepository,
    DC: DriverCacheRepository,
    W: WorkdayRepository,
    M: MailRepository,
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
            .driver_repository
            .get_actual_driver_limitation()
            .await?;
        if let Some(limitation_info) = limitation {
            let current_drivers = self.driver_repository.get_number_of_drivers().await?;
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

        let driver = self.driver_repository.create_driver(create_request).await?;

        let verify_value = self
            .driver_cache_repository
            .generate_random_value(32)
            .await?;
        let redis_key = self
            .driver_cache_repository
            .get_key_by_type(driver.pk_driver_id, DriverCacheKeyType::VerifyEmail);
        let _ = self
            .driver_cache_repository
            .set_redis(redis_key, verify_value.clone(), 3600)
            .await;

        self.mail_repository
            .send_driver_creation_email(driver.clone())
            .await
            .map_err(|_| DriverError::Internal)?;

        Ok(driver)
    }

    async fn login_driver(
        &self,
        login_request: LoginDriverRequest,
    ) -> Result<DriverRow, DriverError> {
        let email = login_request.email.trim().to_lowercase();
        let driver = self
            .driver_repository
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
            .driver_repository
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

    async fn generate_tokens<F>(
        &self,
        mut driver: DriverRow,
        create_tokens: F,
        refresh_ttl: u64,
    ) -> Result<(String, String), DriverError>
    where
        F: Fn(&DriverRow) -> Result<(String, String), DriverError> + Send + Sync,
    {
        let (access_token, refresh_token) = create_tokens(&driver)?;
        let refresh_token_cookie = format!(
            "refresh_token={}; Path=/; Domain=.plannify.be; HttpOnly; Secure; SameSite=None; Max-Age={}",
            refresh_token, refresh_ttl
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
        self.driver_repository.update_driver(driver).await?;

        Ok((access_token, refresh_token_cookie))
    }

    async fn get_driver_rest_periods(
        &self,
        driver_id: Uuid,
    ) -> Result<Vec<DriverRestPeriod>, DriverError> {
        let rest_periods = self
            .driver_repository
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

        self.driver_repository
            .set_driver_rest_periods(driver_id, rest_periods_service)
            .await
    }

    async fn delete_driver_rest_periods(&self, driver_id: Uuid) -> Result<(), DriverError> {
        self.driver_repository
            .delete_driver_rest_periods(driver_id)
            .await
    }
}
