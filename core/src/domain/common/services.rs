use crate::domain::{
    document::port::DocumentExternalRepository,
    driver::port::{DriverCacheRepository, DriverDatabaseRepository},
    health::port::HealthRepository,
    mail::port::{MailDatabaseRepository, MailSmtpRepository},
    update::port::{UpdateCacheRepository, UpdateDatabaseRepository},
    workday::port::{WorkdayCacheRepository, WorkdayDatabaseRepository},
};

#[derive(Clone)]
pub struct Service<H, DD, DC, WD, WC, MS, MD, UD, UC, DE>
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
    pub(crate) health_repository: H,
    pub(crate) driver_database_repository: DD,
    pub(crate) driver_cache_repository: DC,
    pub(crate) workday_database_repository: WD,
    pub(crate) workday_cache_repository: WC,
    pub(crate) mail_smtp_repository: MS,
    pub(crate) mail_database_repository: MD,
    pub(crate) update_database_repository: UD,
    pub(crate) update_cache_repository: UC,
    pub(crate) document_external_repository: DE,
}

#[allow(clippy::too_many_arguments)]
impl<H, DD, DC, WD, WC, MS, MD, UD, UC, DE> Service<H, DD, DC, WD, WC, MS, MD, UD, UC, DE>
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
    pub fn new(
        health_repository: H,
        driver_database_repository: DD,
        driver_cache_repository: DC,
        workday_database_repository: WD,
        workday_cache_repository: WC,
        mail_smtp_repository: MS,
        mail_database_repository: MD,
        update_database_repository: UD,
        update_cache_repository: UC,
        document_external_repository: DE,
    ) -> Self {
        Self {
            health_repository,
            driver_database_repository,
            driver_cache_repository,
            workday_database_repository,
            workday_cache_repository,
            mail_smtp_repository,
            mail_database_repository,
            update_database_repository,
            update_cache_repository,
            document_external_repository,
        }
    }
}
