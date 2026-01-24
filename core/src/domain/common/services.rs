use crate::domain::{
    driver::port::{DriverCacheRepository, DriverRepository},
    health::port::HealthRepository,
    mail::port::MailSmtpRepository,
    workday::port::WorkdayRepository,
};

#[derive(Clone)]
pub struct Service<H, D, DC, W, M>
where
    H: HealthRepository,
    D: DriverRepository,
    DC: DriverCacheRepository,
    W: WorkdayRepository,
    M: MailSmtpRepository,
{
    pub(crate) health_repository: H,
    pub(crate) driver_repository: D,
    pub(crate) driver_cache_repository: DC,
    pub(crate) workday_repository: W,
    pub(crate) mail_smtp_repository: M,
}

impl<H, D, DC, W, M> Service<H, D, DC, W, M>
where
    H: HealthRepository,
    D: DriverRepository,
    DC: DriverCacheRepository,
    W: WorkdayRepository,
    M: MailSmtpRepository,
{
    pub fn new(
        health_repository: H,
        driver_repository: D,
        driver_cache_repository: DC,
        workday_repository: W,
        mail_smtp_repository: M,
    ) -> Self {
        Self {
            health_repository,
            driver_repository,
            driver_cache_repository,
            workday_repository,
            mail_smtp_repository,
        }
    }
}
