use crate::domain::{
    driver::port::{DriverCacheRepository, DriverRepository},
    health::port::HealthRepository,
    mail::port::{MailDatabaseRepository, MailSmtpRepository},
    update::port::{UpdateCacheRepository, UpdateDatabaseRepository},
    workday::port::WorkdayRepository,
};

#[derive(Clone)]
pub struct Service<H, D, DC, W, MS, MD, UD, UC>
where
    H: HealthRepository,
    D: DriverRepository,
    DC: DriverCacheRepository,
    W: WorkdayRepository,
    MS: MailSmtpRepository,
    MD: MailDatabaseRepository,
    UD: UpdateDatabaseRepository,
    UC: UpdateCacheRepository,
{
    pub(crate) health_repository: H,
    pub(crate) driver_repository: D,
    pub(crate) driver_cache_repository: DC,
    pub(crate) workday_repository: W,
    pub(crate) mail_smtp_repository: MS,
    pub(crate) mail_database_repository: MD,
    pub(crate) update_database_repository: UD,
    pub(crate) update_cache_repository: UC,
}

#[allow(clippy::too_many_arguments)]
impl<H, D, DC, W, MS, MD, UD, UC> Service<H, D, DC, W, MS, MD, UD, UC>
where
    H: HealthRepository,
    D: DriverRepository,
    DC: DriverCacheRepository,
    W: WorkdayRepository,
    MS: MailSmtpRepository,
    MD: MailDatabaseRepository,
    UD: UpdateDatabaseRepository,
    UC: UpdateCacheRepository,
{
    pub fn new(
        health_repository: H,
        driver_repository: D,
        driver_cache_repository: DC,
        workday_repository: W,
        mail_smtp_repository: MS,
        mail_database_repository: MD,
        update_database_repository: UD,
        update_cache_repository: UC,
    ) -> Self {
        Self {
            health_repository,
            driver_repository,
            driver_cache_repository,
            workday_repository,
            mail_smtp_repository,
            mail_database_repository,
            update_database_repository,
            update_cache_repository,
        }
    }
}
