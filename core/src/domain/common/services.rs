use crate::domain::{
    driver::port::{DriverCacheRepository, DriverRepository},
    health::port::HealthRepository,
    mail::port::{MailDatabaseRepository, MailSmtpRepository},
    workday::port::WorkdayRepository,
};

#[derive(Clone)]
pub struct Service<H, D, DC, W, MS, MD>
where
    H: HealthRepository,
    D: DriverRepository,
    DC: DriverCacheRepository,
    W: WorkdayRepository,
    MS: MailSmtpRepository,
    MD: MailDatabaseRepository,
{
    pub(crate) health_repository: H,
    pub(crate) driver_repository: D,
    pub(crate) driver_cache_repository: DC,
    pub(crate) workday_repository: W,
    pub(crate) mail_smtp_repository: MS,
    pub(crate) mail_database_repository: MD,
}

impl<H, D, DC, W, MS, MD> Service<H, D, DC, W, MS, MD>
where
    H: HealthRepository,
    D: DriverRepository,
    DC: DriverCacheRepository,
    W: WorkdayRepository,
    MS: MailSmtpRepository,
    MD: MailDatabaseRepository,
{
    pub fn new(
        health_repository: H,
        driver_repository: D,
        driver_cache_repository: DC,
        workday_repository: W,
        mail_smtp_repository: MS,
        mail_database_repository: MD,
    ) -> Self {
        Self {
            health_repository,
            driver_repository,
            driver_cache_repository,
            workday_repository,
            mail_smtp_repository,
            mail_database_repository,
        }
    }
}
