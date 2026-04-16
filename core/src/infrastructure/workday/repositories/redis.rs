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

    #[tracing::instrument(
        name = "cache.workdays.delete_key",
        skip(self),
        fields(
            driver_id = %driver_id,
            prefix = %prefix,
        )
    )]
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

        let Some(json) = json_string else {
            return Ok(None);
        };

        match serde_json::from_str(&json) {
            Ok(workdays) => Ok(workdays),
            Err(e) => {
                error!(
                    "Failed to deserialize workdays from key {}, treating as cache miss: {:?}",
                    key, e
                );
                Ok(None)
            }
        }
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

        let Some(json) = json_string else {
            return Ok(None);
        };

        match serde_json::from_str(&json) {
            Ok(workdays_and_count) => Ok(Some(workdays_and_count)),
            Err(e) => {
                error!(
                    "Failed to deserialize workdays and count from key {}, treating as cache miss: {:?}",
                    key, e
                );
                Ok(None)
            }
        }
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
    ) -> Result<Option<Option<WorkdayDocument>>, WorkdayError> {
        let mut conn = self.connection.clone();
        let (key, _) =
            self.get_key_by_type(driver_id, WorkdayCacheKeyType::Document { month, year });

        let json_string: Option<String> = conn.get(key.clone()).await.map_err(|e| {
            error!("Failed to get redis key {}: {:?}", key, e);
            WorkdayError::Internal
        })?;

        let Some(json) = json_string else {
            return Ok(None); // cache miss
        };

        match serde_json::from_str(&json) {
            Ok(record) => Ok(Some(record)), // Some(None) = absence, Some(Some(doc)) = hit
            Err(e) => {
                error!(
                    "Failed to deserialize workday document record from key {}, treating as cache miss: {:?}",
                    key, e
                );
                Ok(None)
            }
        }
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
        let (key, ttl) =
            self.get_key_by_type(driver_id, WorkdayCacheKeyType::Document { month, year });

        let json_string = serde_json::to_string(&record).map_err(|e| {
            error!("Failed to serialize workday document record: {:?}", e);
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
        name = "cache.workdays.get_document_years",
        skip(self),
        fields(
            db.system = "redis",
            db.operation = "GET",
            driver_id = %driver_id,
        )
    )]
    async fn get_document_years(&self, driver_id: Uuid) -> Result<Option<Vec<i32>>, WorkdayError> {
        let mut conn = self.connection.clone();
        let (key, _) = self.get_key_by_type(driver_id, WorkdayCacheKeyType::DocumentYears);

        let json_string: Option<String> = conn.get(key.clone()).await.map_err(|e| {
            error!("Failed to get redis key {}: {:?}", key, e);
            WorkdayError::Internal
        })?;

        let Some(json) = json_string else {
            return Ok(None);
        };

        match serde_json::from_str(&json) {
            Ok(years) => Ok(Some(years)),
            Err(e) => {
                error!(
                    "Failed to deserialize document years from key {}, treating as cache miss: {:?}",
                    key, e
                );
                Ok(None)
            }
        }
    }

    #[tracing::instrument(
        name = "cache.workdays.set_document_years",
        skip(self),
        fields(
            db.system = "redis",
            db.operation = "SET",
            driver_id = %driver_id,
        )
    )]
    async fn set_document_years(
        &self,
        driver_id: Uuid,
        years: Vec<i32>,
    ) -> Result<(), WorkdayError> {
        let mut conn = self.connection.clone();
        let (key, ttl) = self.get_key_by_type(driver_id, WorkdayCacheKeyType::DocumentYears);

        let json_string = serde_json::to_string(&years).map_err(|e| {
            error!("Failed to serialize document years: {:?}", e);
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
        name = "cache.workdays.delete_document_years",
        skip(self),
        fields(
            db.system = "redis",
            db.operation = "DEL",
            driver_id = %driver_id,
        )
    )]
    async fn delete_document_years(&self, driver_id: Uuid) -> Result<(), WorkdayError> {
        let mut conn = self.connection.clone();
        let (key, _) = self.get_key_by_type(driver_id, WorkdayCacheKeyType::DocumentYears);

        let _: () = conn.del(key.clone()).await.map_err(|e| {
            error!("Failed to delete redis key {}: {:?}", key, e);
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

        match serde_json::from_str(&json) {
            Ok(documents) => Ok(Some(documents)),
            Err(e) => {
                error!(
                    "Failed to deserialize documents by year from key {}, treating as cache miss: {:?}",
                    key, e
                );
                Ok(None)
            }
        }
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
