use chrono::Utc;

use crate::{
    Service,
    domain::{
        document::port::DocumentExternalRepository,
        driver::port::{DriverCacheRepository, DriverDatabaseRepository},
        driver_information::{
            entities::DriverInformation,
            port::{
                DriverInformationCacheRepository, DriverInformationDatabaseRepository,
                DriverInformationService,
            },
        },
        health::port::HealthRepository,
        mail::port::{MailCacheRepository, MailDatabaseRepository, MailSmtpRepository},
        storage::port::StorageRepository,
        update::port::{UpdateCacheRepository, UpdateDatabaseRepository},
        workday::port::{WorkdayCacheRepository, WorkdayDatabaseRepository},
    },
    infrastructure::driver_information::repositories::error::DriverInformationError,
};

impl<H, DD, DC, WD, WC, MS, MD, MC, UD, UC, DE, DS, DID, DIC> DriverInformationService
    for Service<H, DD, DC, WD, WC, MS, MD, MC, UD, UC, DE, DS, DID, DIC>
where
    H: HealthRepository,
    DD: DriverDatabaseRepository,
    DC: DriverCacheRepository,
    WD: WorkdayDatabaseRepository,
    WC: WorkdayCacheRepository,
    MS: MailSmtpRepository,
    MD: MailDatabaseRepository,
    MC: MailCacheRepository,
    UD: UpdateDatabaseRepository,
    UC: UpdateCacheRepository,
    DE: DocumentExternalRepository,
    DS: StorageRepository,
    DID: DriverInformationDatabaseRepository,
    DIC: DriverInformationCacheRepository,
{
    #[tracing::instrument(
        name = "driver_information_service.get_driver_informations",
        skip(self)
    )]
    async fn get_driver_informations(
        &self,
    ) -> Result<Vec<DriverInformation>, DriverInformationError> {
        let cached = if let Some(cached) = self
            .driver_information_cache_repository
            .get_driver_informations()
            .await?
        {
            cached
        } else {
            let rows = self
                .driver_information_database_repository
                .get_driver_informations()
                .await?;

            let cache_items: Vec<_> = rows.iter().map(|r| r.to_cache()).collect();

            let _ = self
                .driver_information_cache_repository
                .set_driver_informations(cache_items.clone())
                .await;

            cache_items
        };

        let now = Utc::now();

        Ok(cached
            .into_iter()
            .filter(|information| information.show_at <= now)
            .map(|information| information.to_driver_information())
            .collect())
    }
}
