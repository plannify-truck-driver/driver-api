use crate::{
    Service,
    domain::{
        document::port::DocumentExternalRepository,
        driver::port::{DriverCacheRepository, DriverDatabaseRepository},
        health::{
            entities::IsHealthy,
            port::{HealthRepository, HealthService},
        },
        mail::port::{MailDatabaseRepository, MailSmtpRepository},
        update::port::{UpdateCacheRepository, UpdateDatabaseRepository},
        workday::port::{WorkdayCacheRepository, WorkdayDatabaseRepository},
    },
    infrastructure::health::repositories::error::HealthError,
};

impl<H, DD, DC, WD, WC, MS, MD, UD, UC, DE> HealthService
    for Service<H, DD, DC, WD, WC, MS, MD, UD, UC, DE>
where
    H: HealthRepository,
    DD: DriverDatabaseRepository,
    DC: DriverCacheRepository,
    WD: WorkdayDatabaseRepository,
    WC: WorkdayCacheRepository,
    MS: MailSmtpRepository,
    MD: MailDatabaseRepository,
    UD: UpdateDatabaseRepository,
    UC: UpdateCacheRepository,
    DE: DocumentExternalRepository,
{
    async fn check_health(&self) -> Result<IsHealthy, HealthError> {
        self.health_repository.ping().await.to_result()
    }
}
