use crate::{
    Service,
    domain::{
        driver::{
            entities::{CreateDriverRequest, DriverRow, LoginDriverRequest},
            port::{DriverRepository, DriverService},
        },
        health::port::HealthRepository,
    },
    infrastructure::driver::repositories::error::DriverError,
};
use argon2::{
    Algorithm, Argon2, Params, Version,
    password_hash::PasswordHash,
    password_hash::{PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use tracing::error;

impl<H, D> DriverService for Service<H, D>
where
    H: HealthRepository,
    D: DriverRepository,
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
        create_request.firstname = Self::to_title_case(create_request.firstname);
        create_request.lastname = Self::to_title_case(create_request.lastname);
        create_request.email = create_request.email.trim().to_lowercase();

        let email_domain = create_request.email.split('@').last().unwrap_or("");

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

        self.driver_repository.create_driver(create_request).await
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

        Ok(driver)
    }

    async fn generate_tokens<F>(
        &self,
        mut driver: DriverRow,
        create_tokens: F,
        refresh_ttl: u64,
    ) -> Result<(String, String), DriverError>
    where
        F: Fn(uuid::Uuid) -> Result<(String, String), DriverError> + Send + Sync,
    {
        let (access_token, refresh_token) = create_tokens(driver.pk_driver_id)?;
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
}
