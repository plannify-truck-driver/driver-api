use crate::domain::{
    driver::port::DriverRepository, health::port::HealthRepository, mail::port::MailRepository,
    workday::port::WorkdayRepository,
};

#[derive(Clone)]
pub struct Service<H, D, W, M>
where
    H: HealthRepository,
    D: DriverRepository,
    W: WorkdayRepository,
    M: MailRepository,
{
    pub(crate) health_repository: H,
    pub(crate) driver_repository: D,
    pub(crate) workday_repository: W,
    pub(crate) mail_repository: M,
}

impl<H, D, W, M> Service<H, D, W, M>
where
    H: HealthRepository,
    D: DriverRepository,
    W: WorkdayRepository,
    M: MailRepository,
{
    pub fn new(
        health_repository: H,
        driver_repository: D,
        workday_repository: W,
        mail_repository: M,
    ) -> Self {
        Self {
            health_repository,
            driver_repository,
            workday_repository,
            mail_repository,
        }
    }
}
