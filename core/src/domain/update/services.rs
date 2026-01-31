use crate::{
    Service,
    domain::{
        document::port::DocumentExternalRepository,
        driver::port::{DriverCacheRepository, DriverDatabaseRepository},
        health::port::HealthRepository,
        mail::port::{MailDatabaseRepository, MailSmtpRepository},
        update::{
            entities::UpdateCache,
            port::{UpdateCacheRepository, UpdateDatabaseRepository, UpdateService},
        },
        workday::port::{WorkdayCacheRepository, WorkdayDatabaseRepository},
    },
    infrastructure::update::repositories::error::UpdateError,
};

impl<H, DD, DC, WD, WC, MS, MD, UD, UC, DE> UpdateService
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
    async fn get_updates_by_version(
        &self,
        version: String,
        page: u32,
        limit: u32,
    ) -> Result<(Vec<UpdateCache>, u32), UpdateError> {
        let cached_updates = self
            .update_cache_repository
            .get_updates_by_version(version.clone(), page, limit)
            .await?;
        if let Some(cached_updates) = cached_updates {
            let total = cached_updates.len() as u32;
            return Ok((cached_updates, total));
        }

        let (updates, count) = self
            .update_database_repository
            .get_updates_by_version(version.clone(), page, limit)
            .await?;

        let updates_cache: Vec<UpdateCache> = updates.iter().map(|u| u.to_cache()).collect();

        self.update_cache_repository
            .set_updates_by_version(version.clone(), page, limit, updates_cache.clone())
            .await?;

        Ok((updates_cache, count))
    }
}
