use crate::{
    Service,
    domain::{
        driver::port::DriverRepository,
        health::{
            entities::IsHealthy,
            port::{HealthRepository, HealthService},
        },
        workday::port::WorkdayRepository,
    },
    infrastructure::health::repositories::error::HealthError,
};

impl<H, D, W> HealthService for Service<H, D, W>
where
    H: HealthRepository,
    D: DriverRepository,
    W: WorkdayRepository,
{
    async fn check_health(&self) -> Result<IsHealthy, HealthError> {
        self.health_repository.ping().await.to_result()
    }
}
