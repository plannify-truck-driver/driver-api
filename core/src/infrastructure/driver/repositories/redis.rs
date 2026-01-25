use passwords::PasswordGenerator;
use redis::{AsyncCommands, aio::ConnectionManager};
use tracing::error;
use uuid::Uuid;

use crate::{
    domain::driver::port::DriverCacheRepository,
    infrastructure::driver::repositories::error::DriverError,
};

#[derive(Clone)]
pub struct RedisDriverCacheRepository {
    connection: ConnectionManager,
}

impl RedisDriverCacheRepository {
    pub fn new(connection: ConnectionManager) -> Self {
        Self { connection }
    }
}

impl DriverCacheRepository for RedisDriverCacheRepository {
    async fn generate_random_value(&self, length: usize) -> Result<String, DriverError> {
        let generator = PasswordGenerator {
            length,
            numbers: true,
            lowercase_letters: true,
            uppercase_letters: true,
            symbols: false,
            spaces: false,
            exclude_similar_characters: false,
            strict: true,
        };

        match generator.generate_one() {
            Ok(key) => Ok(key),
            Err(e) => {
                error!("Failed to generate random key: {:?}", e);
                Err(DriverError::Internal)
            }
        }
    }

    fn generate_redis_key(&self, driver_id: Uuid, suffix: &str) -> String {
        format!("driver:{}:{}", driver_id, suffix)
    }

    async fn set_redis(
        &self,
        key: String,
        value: String,
        ttl_seconds: u64,
    ) -> Result<(), DriverError> {
        let mut conn = self.connection.clone();
        let _: () = conn
            .set_ex(key.clone(), value, ttl_seconds)
            .await
            .map_err(|e| {
                error!("Failed to set redis key {}: {:?}", key, e);
                DriverError::Internal
            })?;

        Ok(())
    }

    async fn get_redis(&self, key: String) -> Result<Option<String>, DriverError> {
        let mut conn = self.connection.clone();
        let result: Option<String> = conn.get(key.clone()).await.map_err(|e| {
            error!("Failed to get redis key {}: {:?}", key, e);
            DriverError::Internal
        })?;

        Ok(result)
    }
}
