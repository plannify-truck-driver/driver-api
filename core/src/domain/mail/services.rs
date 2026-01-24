use chrono::Utc;

use crate::{
    Service,
    domain::{
        driver::{
            entities::DriverRow,
            port::{DriverCacheKeyType, DriverCacheRepository, DriverRepository},
        },
        health::port::HealthRepository,
        mail::{
            entities::MailStatus,
            port::{MailDatabaseRepository, MailService, MailSmtpRepository},
        },
        workday::port::WorkdayRepository,
    },
    infrastructure::mail::repositories::error::MailError,
};

impl<H, D, DC, W, MS, MD> MailService for Service<H, D, DC, W, MS, MD>
where
    H: HealthRepository,
    D: DriverRepository,
    DC: DriverCacheRepository,
    W: WorkdayRepository,
    MS: MailSmtpRepository,
    MD: MailDatabaseRepository,
{
    async fn send_creation_email(&self, driver: DriverRow) -> Result<(), MailError> {
        let verify_value = self
            .driver_cache_repository
            .generate_random_value(32)
            .await
            .map_err(|_| MailError::Internal)?;

        let verify_ttl = 3600;

        let redis_key = self
            .driver_cache_repository
            .get_key_by_type(driver.pk_driver_id, DriverCacheKeyType::VerifyEmail);
        let _ = self
            .driver_cache_repository
            .set_redis(redis_key, verify_value.clone(), verify_ttl)
            .await;

        let mail = self
            .mail_database_repository
            .create_mail(
                driver.clone(),
                1, // Assuming 1 is the mail type ID for creation emails
                "Driver account creation".to_string(),
                Some(format!(
                    "Welcome! Please verify your email using this code: {}",
                    verify_value
                )),
            )
            .await?;

        match self
            .mail_smtp_repository
            .send_driver_creation_email(driver.clone(), verify_value, verify_ttl)
            .await
        {
            Ok(_) => {
                self.mail_database_repository
                    .update_mail_status(
                        mail.pk_driver_mail_id,
                        MailStatus::SUCCESS,
                        Some(Utc::now()),
                    )
                    .await?;
            }
            Err(_) => {
                let _ = self
                    .mail_database_repository
                    .update_mail_status(mail.pk_driver_mail_id, MailStatus::FAILED, None)
                    .await?;

                return Err(MailError::Internal);
            }
        }

        Ok(())
    }
}
