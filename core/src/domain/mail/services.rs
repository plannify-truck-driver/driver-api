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
            .generate_random_value(100)
            .await
            .map_err(|_| MailError::Internal)?;

        let (redis_key, redis_ttl) = self
            .driver_cache_repository
            .get_key_by_type(driver.pk_driver_id, DriverCacheKeyType::VerifyEmail);
        let _ = self
            .driver_cache_repository
            .set_redis(redis_key, verify_value.clone(), redis_ttl)
            .await;

        let mail = self
            .mail_database_repository
            .create_mail(
                driver.clone(),
                1,
                "Driver account creation".to_string(),
                None,
            )
            .await?;

        match self
            .mail_smtp_repository
            .send_driver_creation_email(driver.clone(), verify_value, redis_ttl)
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
