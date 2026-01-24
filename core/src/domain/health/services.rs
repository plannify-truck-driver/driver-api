use crate::{
    Service,
    domain::{
        driver::port::{DriverCacheRepository, DriverRepository},
        health::{
            entities::IsHealthy,
            port::{HealthRepository, HealthService},
        },
        mail::port::MailSmtpRepository,
        workday::port::WorkdayRepository,
    },
    infrastructure::health::repositories::error::HealthError,
};

impl<H, D, DC, W, E> HealthService for Service<H, D, DC, W, E>
where
    H: HealthRepository,
    D: DriverRepository,
    DC: DriverCacheRepository,
    W: WorkdayRepository,
    E: MailSmtpRepository,
{
    async fn check_health(&self) -> Result<IsHealthy, HealthError> {
        self.health_repository.ping().await.to_result()
    }
}
