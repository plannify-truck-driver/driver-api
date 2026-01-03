use crate::{
    Service,
    domain::{
        driver::port::DriverRepository,
        health::{
            entities::IsHealthy,
            port::{HealthRepository, HealthService},
        },
    },
    infrastructure::health::repositories::error::HealthError,
};

impl<H, D> HealthService for Service<H, D>
where
    H: HealthRepository,
    D: DriverRepository,
{
    async fn check_health(&self) -> Result<IsHealthy, HealthError> {
        self.health_repository.ping().await.to_result()
    }
}
