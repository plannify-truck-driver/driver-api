use redis::{AsyncCommands, aio::ConnectionManager};
use serde_json;
use uuid::Uuid;

use crate::{
    domain::workday::{
        entities::{Workday, WorkdayDocument, WorkdayDocumentInformation},
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

    async fn delete_key(&self, driver_id: Uuid, prefix: &str) -> Result<(), WorkdayError> {
        let mut conn = self.connection.clone();
        let key_pattern = format!("driver:{}:{}*", driver_id, prefix);
        let keys: Vec<String> = conn.keys(key_pattern.clone()).await.map_err(|e| {
            error!("Failed to get keys for pattern {}: {:?}", key_pattern, e);
            WorkdayError::Internal
        })?;

        if !keys.is_empty() {
            let _: () = conn.del(keys).await.map_err(|e| {
                error!("Failed to delete keys for pattern {}: {:?}", key_pattern, e);
                WorkdayError::Internal
            })?;
        }

        Ok(())
    }

    #[tracing::instrument(
        name = "cache.workdays.get_workdays_by_month",
        skip(self),
        fields(
            driver_id = %driver_id,
            month = %month,
            year = %year,
        )
    )]
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

    #[tracing::instrument(
        name = "cache.workdays.set_workdays_by_month",
        skip(self),
        fields(
            driver_id = %driver_id,
            month = %month,
            year = %year,
        )
    )]
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

    #[tracing::instrument(
        name = "cache.workdays.delete_workdays_by_month",
        skip(self),
        fields(
            driver_id = %driver_id,
            month = %month,
            year = %year,
        )
    )]
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

    #[tracing::instrument(
        name = "cache.workdays.get_workdays_by_month",
        skip(self),
        fields(
            driver_id = %driver_id,
            start_date = %start_date,
            end_date = %end_date,
            page = %page,
            limit = %limit,
        )
    )]
    async fn get_workdays_by_period(
        &self,
        driver_id: Uuid,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
        page: u32,
        limit: u32,
    ) -> Result<Option<(Vec<Workday>, u32)>, WorkdayError> {
        let mut conn = self.connection.clone();
        let (key, _) = self.get_key_by_type(
            driver_id,
            WorkdayCacheKeyType::Period {
                start_date,
                end_date,
                page,
                limit,
            },
        );

        let json_string: Option<String> = conn.get(key.clone()).await.map_err(|e| {
            error!("Failed to get redis key {}: {:?}", key, e);
            WorkdayError::Internal
        })?;

        if json_string.is_none() {
            return Ok(None);
        }

        let workdays_and_count: (Vec<Workday>, u32) = serde_json::from_str(&json_string.unwrap())
            .map_err(|e| {
            error!("Failed to deserialize workdays and count: {:?}", e);
            WorkdayError::Internal
        })?;

        Ok(Some(workdays_and_count))
    }

    #[tracing::instrument(
        name = "cache.workdays.set_workdays_by_period",
        skip(self),
        fields(
            driver_id = %driver_id,
            start_date = %start_date,
            end_date = %end_date,
            page = %page,
            limit = %limit,
        )
    )]
    async fn set_workdays_by_period(
        &self,
        driver_id: Uuid,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
        page: u32,
        limit: u32,
        workdays: Vec<Workday>,
        total_count: u32,
    ) -> Result<(), WorkdayError> {
        let mut conn = self.connection.clone();
        let (key, ttl) = self.get_key_by_type(
            driver_id,
            WorkdayCacheKeyType::Period {
                start_date,
                end_date,
                page,
                limit,
            },
        );

        let workdays_and_count = (workdays, total_count);
        let json_string = serde_json::to_string(&workdays_and_count).map_err(|e| {
            error!("Failed to serialize workdays and count: {:?}", e);
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

    #[tracing::instrument(
        name = "cache.workdays.delete_workdays_by_period",
        skip(self),
        fields(
            driver_id = %driver_id,
            start_date = %start_date,
            end_date = %end_date,
            page = %page,
            limit = %limit,
        )
    )]
    async fn delete_workdays_by_period(
        &self,
        driver_id: Uuid,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
        page: u32,
        limit: u32,
    ) -> Result<(), WorkdayError> {
        let mut conn = self.connection.clone();
        let (key, _) = self.get_key_by_type(
            driver_id,
            WorkdayCacheKeyType::Period {
                start_date,
                end_date,
                page,
                limit,
            },
        );

        let _: () = conn.del(key.clone()).await.map_err(|e| {
            error!("Failed to delete redis key {}: {:?}", key, e);
            WorkdayError::Internal
        })?;

        Ok(())
    }

    #[tracing::instrument(
        name = "cache.workdays.get_workday_document_record",
        skip(self),
        fields(
            db.system = "redis",
            db.operation = "GET",
            driver_id = %driver_id,
            month = %month,
            year = %year,
        )
    )]
    async fn get_workday_document_record(
        &self,
        driver_id: Uuid,
        month: i32,
        year: i32,
    ) -> Result<Option<WorkdayDocument>, WorkdayError> {
        let mut conn = self.connection.clone();
        let (key, _) = self.get_key_by_type(
            driver_id,
            WorkdayCacheKeyType::Document { month, year },
        );

        let json_string: Option<String> = conn.get(key.clone()).await.map_err(|e| {
            error!("Failed to get redis key {}: {:?}", key, e);
            WorkdayError::Internal
        })?;

        let Some(json) = json_string else {
            return Ok(None);
        };

        let record: Option<WorkdayDocument> = serde_json::from_str(&json).map_err(|e| {
            error!("Failed to deserialize workday document record: {:?}", e);
            WorkdayError::Internal
        })?;

        Ok(record)
    }

    #[tracing::instrument(
        name = "cache.workdays.set_workday_document_record",
        skip(self, record),
        fields(
            db.system = "redis",
            db.operation = "SET",
            driver_id = %driver_id,
            month = %month,
            year = %year,
        )
    )]
    async fn set_workday_document_record(
        &self,
        driver_id: Uuid,
        month: i32,
        year: i32,
        record: Option<WorkdayDocument>,
    ) -> Result<(), WorkdayError> {
        let mut conn = self.connection.clone();
        let (key, ttl) = self.get_key_by_type(
            driver_id,
            WorkdayCacheKeyType::Document { month, year },
        );

        let json_string = serde_json::to_string(&record).map_err(|e| {
            error!("Failed to serialize workday document record: {:?}", e);
            WorkdayError::Internal
        })?;

        let _: () = conn.set_ex(key.clone(), json_string, ttl).await.map_err(|e| {
            error!("Failed to set redis key {}: {:?}", key, e);
            WorkdayError::Internal
        })?;

        Ok(())
    }

    #[tracing::instrument(
        name = "cache.workdays.get_documents_by_year",
        skip(self),
        fields(
            db.system = "redis",
            db.operation = "GET",
            driver_id = %driver_id,
            year = %year,
        )
    )]
    async fn get_documents_by_year(
        &self,
        driver_id: Uuid,
        year: i32,
    ) -> Result<Option<Vec<WorkdayDocumentInformation>>, WorkdayError> {
        let mut conn = self.connection.clone();
        let (key, _) =
            self.get_key_by_type(driver_id, WorkdayCacheKeyType::DocumentsByYear { year });

        let json_string: Option<String> = conn.get(key.clone()).await.map_err(|e| {
            error!("Failed to get redis key {}: {:?}", key, e);
            WorkdayError::Internal
        })?;

        let Some(json) = json_string else {
            return Ok(None);
        };

        let documents = serde_json::from_str(&json).map_err(|e| {
            error!("Failed to deserialize documents by year: {:?}", e);
            WorkdayError::Internal
        })?;

        Ok(Some(documents))
    }

    #[tracing::instrument(
        name = "cache.workdays.set_documents_by_year",
        skip(self, documents),
        fields(
            db.system = "redis",
            db.operation = "SET",
            driver_id = %driver_id,
            year = %year,
        )
    )]
    async fn set_documents_by_year(
        &self,
        driver_id: Uuid,
        year: i32,
        documents: Vec<WorkdayDocumentInformation>,
    ) -> Result<(), WorkdayError> {
        let mut conn = self.connection.clone();
        let (key, ttl) =
            self.get_key_by_type(driver_id, WorkdayCacheKeyType::DocumentsByYear { year });

        let json_string = serde_json::to_string(&documents).map_err(|e| {
            error!("Failed to serialize documents by year: {:?}", e);
            WorkdayError::Internal
        })?;

        let _: () = conn.set_ex(key.clone(), json_string, ttl).await.map_err(|e| {
            error!("Failed to set redis key {}: {:?}", key, e);
            WorkdayError::Internal
        })?;

        Ok(())
    }

    #[tracing::instrument(
        name = "cache.workdays.delete_documents_by_year",
        skip(self),
        fields(
            db.system = "redis",
            db.operation = "DEL",
            driver_id = %driver_id,
            year = %year,
        )
    )]
    async fn delete_documents_by_year(
        &self,
        driver_id: Uuid,
        year: i32,
    ) -> Result<(), WorkdayError> {
        let mut conn = self.connection.clone();
        let (key, _) =
            self.get_key_by_type(driver_id, WorkdayCacheKeyType::DocumentsByYear { year });

        let _: () = conn.del(key.clone()).await.map_err(|e| {
            error!("Failed to delete redis key {}: {:?}", key, e);
            WorkdayError::Internal
        })?;

        Ok(())
    }
}
