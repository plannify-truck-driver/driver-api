use crate::{
    Service,
    domain::{
        driver::port::{DriverCacheRepository, DriverRepository},
        health::port::HealthRepository,
        mail::port::{MailDatabaseRepository, MailSmtpRepository},
        update::{
            entities::UpdateCache,
            port::{UpdateCacheRepository, UpdateDatabaseRepository, UpdateService},
        },
        workday::port::WorkdayRepository,
    },
    infrastructure::update::repositories::error::UpdateError,
};

impl<H, D, DC, W, MS, MD, UD, UC> UpdateService for Service<H, D, DC, W, MS, MD, UD, UC>
where
    H: HealthRepository,
    D: DriverRepository,
    DC: DriverCacheRepository,
    W: WorkdayRepository,
    MS: MailSmtpRepository,
    MD: MailDatabaseRepository,
    UD: UpdateDatabaseRepository,
    UC: UpdateCacheRepository,
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
