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

    #[tracing::instrument(
        name = "cache.drivers.set_redis",
        skip(self),
        fields(
            db.system = "redis",
            db.operation = "SET",
        )
    )]
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

    #[tracing::instrument(
        name = "cache.drivers.get_redis",
        skip(self),
        fields(
            db.system = "redis",
            db.operation = "GET",
        )
    )]
    async fn get_redis(&self, key: String) -> Result<Option<String>, DriverError> {
        let mut conn = self.connection.clone();
        let result: Option<String> = conn.get(key.clone()).await.map_err(|e| {
            error!("Failed to get redis key {}: {:?}", key, e);
            DriverError::Internal
        })?;

        Ok(result)
    }

    #[tracing::instrument(
        name = "cache.drivers.delete_redis",
        skip(self),
        fields(
            db.system = "redis",
            db.operation = "DEL",
        )
    )]
    async fn delete_redis(&self, key: String) -> Result<(), DriverError> {
        let mut conn = self.connection.clone();
        let _: () = conn.del(key.clone()).await.map_err(|e| {
            error!("Failed to delete redis key {}: {:?}", key, e);
            DriverError::Internal
        })?;

        Ok(())
    }

    #[tracing::instrument(
        name = "cache.drivers.decrement_redis",
        skip(self),
        fields(
            db.system = "redis",
            db.operation = "DECR",
        )
    )]
    async fn decrement_redis(
        &self,
        key: String,
        initial_value: i64,
        ttl_seconds: u64,
    ) -> Result<i64, DriverError> {
        let mut conn = self.connection.clone();

        // Only takes effect the first time the key is used within a window,
        // so subsequent decrements never push back the window's expiry.
        let _: Option<String> = redis::cmd("SET")
            .arg(&key)
            .arg(initial_value)
            .arg("EX")
            .arg(ttl_seconds)
            .arg("NX")
            .query_async(&mut conn)
            .await
            .map_err(|e| {
                error!("Failed to initialize redis counter {}: {:?}", key, e);
                DriverError::Internal
            })?;

        let result: i64 = conn.decr(key.clone(), 1).await.map_err(|e| {
            error!("Failed to decrement redis counter {}: {:?}", key, e);
            DriverError::Internal
        })?;

        Ok(result)
    }

    #[tracing::instrument(
        name = "cache.drivers.get_ttl",
        skip(self),
        fields(
            db.system = "redis",
            db.operation = "TTL",
        )
    )]
    async fn get_ttl(&self, key: String) -> Result<Option<i64>, DriverError> {
        let mut conn = self.connection.clone();
        let ttl: i64 = conn.ttl(key.clone()).await.map_err(|e| {
            error!("Failed to read TTL for redis key {}: {:?}", key, e);
            DriverError::Internal
        })?;

        // Redis returns -2 when the key doesn't exist and -1 when it exists
        // without an expiry; both mean "no meaningful TTL" here.
        Ok(if ttl >= 0 { Some(ttl) } else { None })
    }
}
