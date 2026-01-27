use redis::{AsyncCommands, aio::ConnectionManager};
use serde_json;
use uuid::Uuid;

use crate::{
    domain::workday::{
        entities::Workday,
        port::{WorkdayCacheKeyType, WorkdayCacheRepository},
    },
    infrastructure::workday::repositories::error::WorkdayError,
};

use tracing::error;

#[derive(Clone)]
pub struct RedisWorkdayRepository {
    connection: ConnectionManager,
}

impl RedisWorkdayRepository {
    pub fn new(connection: ConnectionManager) -> Self {
        Self { connection }
    }
}

impl WorkdayCacheRepository for RedisWorkdayRepository {
    fn generate_redis_key(&self, driver_id: Uuid, suffix: &str) -> String {
        format!("driver:{}:workdays:{}", driver_id, suffix)
    }

    async fn get_workdays_by_month(
        &self,
        driver_id: Uuid,
        month: i32,
        year: i32,
    ) -> Result<Option<Vec<Workday>>, WorkdayError> {
        let mut conn = self.connection.clone();
        let (key, _) =
            self.get_key_by_type(driver_id, WorkdayCacheKeyType::Monthly { month, year });

        let json_string: Option<String> = conn.get(key.clone()).await.map_err(|e| {
            error!("Failed to get redis key {}: {:?}", key, e);
            WorkdayError::Internal
        })?;

        if json_string.is_none() {
            return Ok(None);
        }

        let workdays = serde_json::from_str(&json_string.unwrap()).map_err(|e| {
            error!("Failed to deserialize workdays: {:?}", e);
            WorkdayError::Internal
        })?;

        Ok(workdays)
    }

    async fn set_workdays_by_month(
        &self,
        driver_id: Uuid,
        month: i32,
        year: i32,
        workdays: Vec<Workday>,
    ) -> Result<(), WorkdayError> {
        let mut conn = self.connection.clone();
        let (key, ttl) =
            self.get_key_by_type(driver_id, WorkdayCacheKeyType::Monthly { month, year });

        let json_string = serde_json::to_string(&workdays).map_err(|e| {
            error!("Failed to serialize workdays: {:?}", e);
            WorkdayError::Internal
        })?;

        let _: () = conn
            .set_ex(key.clone(), json_string, ttl)
            .await
            .map_err(|e| {
                error!("Failed to set redis key {}: {:?}", key, e);
                WorkdayError::Internal
            })?;

        Ok(())
    }

    async fn delete_workdays_by_month(
        &self,
        driver_id: Uuid,
        month: i32,
        year: i32,
    ) -> Result<(), WorkdayError> {
        let mut conn = self.connection.clone();
        let (key, _) =
            self.get_key_by_type(driver_id, WorkdayCacheKeyType::Monthly { month, year });

        let _: () = conn.del(key.clone()).await.map_err(|e| {
            error!("Failed to delete redis key {}: {:?}", key, e);
            WorkdayError::Internal
        })?;

        Ok(())
    }
}
