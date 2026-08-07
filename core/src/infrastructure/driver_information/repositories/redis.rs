use redis::{AsyncCommands, aio::ConnectionManager};
use tracing::error;

use crate::{
    domain::driver_information::{
        entities::DriverInformationCache, port::DriverInformationCacheRepository,
    },
    infrastructure::driver_information::repositories::error::DriverInformationError,
};

#[derive(Clone)]
pub struct RedisDriverInformationCacheRepository {
    connection: ConnectionManager,
}

impl RedisDriverInformationCacheRepository {
    pub fn new(connection: ConnectionManager) -> Self {
        Self { connection }
    }
}

impl DriverInformationCacheRepository for RedisDriverInformationCacheRepository {
    #[tracing::instrument(
        name = "cache.driver_informations.get_driver_informations",
        skip(self),
        fields(db.system = "redis", db.operation = "GET")
    )]
    async fn get_driver_informations(
        &self,
    ) -> Result<Option<Vec<DriverInformationCache>>, DriverInformationError> {
        let mut conn = self.connection.clone();
        let (key, _) = self.driver_informations_cache_key();

        let json: Option<String> = conn.get(key.clone()).await.map_err(|e| {
            error!("Failed to get redis key {}: {:?}", key, e);
            DriverInformationError::Internal
        })?;

        let Some(json) = json else {
            return Ok(None);
        };

        match serde_json::from_str(&json) {
            Ok(informations) => Ok(Some(informations)),
            Err(e) => {
                error!(
                    "Failed to deserialize driver informations from {}, cache miss: {:?}",
                    key, e
                );
                Ok(None)
            }
        }
    }

    #[tracing::instrument(
        name = "cache.driver_informations.set_driver_informations",
        skip(self, informations),
        fields(db.system = "redis", db.operation = "SET")
    )]
    async fn set_driver_informations(
        &self,
        informations: Vec<DriverInformationCache>,
    ) -> Result<(), DriverInformationError> {
        let mut conn = self.connection.clone();
        let (key, ttl) = self.driver_informations_cache_key();

        let json = serde_json::to_string(&informations).map_err(|e| {
            error!("Failed to serialize driver informations: {:?}", e);
            DriverInformationError::Internal
        })?;

        let _: () = conn.set_ex(key.clone(), json, ttl).await.map_err(|e| {
            error!("Failed to set redis key {}: {:?}", key, e);
            DriverInformationError::Internal
        })?;

        Ok(())
    }
}
