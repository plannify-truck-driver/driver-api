use crate::domain::{driver::port::DriverRepository, health::port::HealthRepository};

#[derive(Clone)]
pub struct Service<H, D>
where
    H: HealthRepository,
    D: DriverRepository
{
    pub(crate) health_repository: H,
    pub(crate) driver_repository: D,
}

impl<H, D> Service<H, D>
where
    H: HealthRepository,
    D: DriverRepository,
{
    pub fn new(health_repository: H, driver_repository: D) -> Self {
        Self { health_repository, driver_repository }
    }
}
