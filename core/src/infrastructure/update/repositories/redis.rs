use redis::{AsyncCommands, aio::ConnectionManager};
use serde_json;

use crate::{
    domain::update::{entities::UpdateCache, port::UpdateCacheRepository},
    infrastructure::update::repositories::error::UpdateError,
};

use tracing::error;

#[derive(Clone)]
pub struct RedisUpdateCacheRepository {
    connection: ConnectionManager,
}

impl RedisUpdateCacheRepository {
    pub fn new(connection: ConnectionManager) -> Self {
        Self { connection }
    }
}

impl UpdateCacheRepository for RedisUpdateCacheRepository {
    fn generate_redis_key(&self, version: String, page: u32, limit: u32) -> String {
        format!("updates:version:{}:page:{}:limit:{}", version, page, limit)
    }

    async fn get_updates_by_version(
        &self,
        version: String,
        page: u32,
        limit: u32,
    ) -> Result<Option<Vec<UpdateCache>>, UpdateError> {
        let mut conn = self.connection.clone();
        let key = self.generate_redis_key(version, page, limit);

        let json_string: Option<String> = conn.get(key.clone()).await.map_err(|e| {
            error!("Failed to get redis key {}: {:?}", key, e);
            UpdateError::Internal
        })?;

        if json_string.is_none() {
            return Ok(None);
        }

        let updates = serde_json::from_str(&json_string.unwrap()).map_err(|e| {
            error!("Failed to deserialize updates: {:?}", e);
            UpdateError::Internal
        })?;

        Ok(updates)
    }

    async fn set_updates_by_version(
        &self,
        version: String,
        page: u32,
        limit: u32,
        updates: Vec<UpdateCache>,
    ) -> Result<(), UpdateError> {
        let mut conn = self.connection.clone();
        let key = self.generate_redis_key(version, page, limit);

        let json_string = serde_json::to_string(&updates).map_err(|e| {
            error!("Failed to serialize updates: {:?}", e);
            UpdateError::Internal
        })?;

        let _: () = conn
            .set_ex(key.clone(), json_string, 3600 * 6)
            .await
            .map_err(|e| {
                error!("Failed to set redis key {}: {:?}", key, e);
                UpdateError::Internal
            })?;

        Ok(())
    }
}
