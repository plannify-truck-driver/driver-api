use crate::domain::{
    driver::port::DriverRepository, health::port::HealthRepository,
    workday::port::WorkdayRepository,
};

#[derive(Clone)]
pub struct Service<H, D, W>
where
    H: HealthRepository,
    D: DriverRepository,
    W: WorkdayRepository,
{
    pub(crate) health_repository: H,
    pub(crate) driver_repository: D,
    pub(crate) workday_repository: W,
}

impl<H, D, W> Service<H, D, W>
where
    H: HealthRepository,
    D: DriverRepository,
    W: WorkdayRepository,
{
    pub fn new(health_repository: H, driver_repository: D, workday_repository: W) -> Self {
        Self {
            health_repository,
            driver_repository,
            workday_repository,
        }
    }
}
