use crate::{
    Service,
    domain::{
        driver::port::{DriverCacheRepository, DriverRepository},
        health::{
            entities::IsHealthy,
            port::{HealthRepository, HealthService},
        },
        mail::port::{MailDatabaseRepository, MailSmtpRepository},
        workday::port::WorkdayRepository,
    },
    infrastructure::health::repositories::error::HealthError,
};

impl<H, D, DC, W, MS, MD> HealthService for Service<H, D, DC, W, MS, MD>
where
    H: HealthRepository,
    D: DriverRepository,
    DC: DriverCacheRepository,
    W: WorkdayRepository,
    MS: MailSmtpRepository,
    MD: MailDatabaseRepository,
{
    async fn check_health(&self) -> Result<IsHealthy, HealthError> {
        self.health_repository.ping().await.to_result()
    }
}
