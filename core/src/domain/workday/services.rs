use chrono::{Datelike, NaiveDate};
use uuid::Uuid;

use crate::{
    Service,
    domain::{
        document::port::DocumentExternalRepository,
        driver::port::{DriverCacheRepository, DriverDatabaseRepository},
        health::port::HealthRepository,
        mail::port::{MailDatabaseRepository, MailSmtpRepository},
        update::port::{UpdateCacheRepository, UpdateDatabaseRepository},
        workday::{
            entities::{
                CreateWorkdayRequest, UpdateWorkdayRequest, Workday, WorkdayGarbageRow, WorkdayRow,
            },
            port::{WorkdayCacheRepository, WorkdayDatabaseRepository, WorkdayService},
        },
    },
    infrastructure::workday::repositories::error::WorkdayError,
};

impl<H, DD, DC, WD, WC, MS, MD, UD, UC, DE> WorkdayService
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
    async fn get_workdays_by_month(
        &self,
        driver_id: Uuid,
        month: i32,
        year: i32,
    ) -> Result<Vec<Workday>, WorkdayError> {
        let cached_workdays = self
            .workday_cache_repository
            .get_workdays_by_month(driver_id, month, year)
            .await?;
        if let Some(cached_workdays) = cached_workdays {
            return Ok(cached_workdays);
        }

        let workdays = self
            .workday_database_repository
            .get_workdays_by_month(driver_id, month, year)
            .await?;
        let workdays_transformed: Vec<Workday> = workdays.iter().map(|w| w.to_workday()).collect();

        self.workday_cache_repository
            .set_workdays_by_month(driver_id, month, year, workdays_transformed.clone())
            .await?;

        Ok(workdays_transformed)
    }

    async fn get_workdays_by_period(
        &self,
        driver_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
        page: u32,
        limit: u32,
    ) -> Result<(Vec<Workday>, u32), WorkdayError> {
        let cached_workdays = self
            .workday_cache_repository
            .get_workdays_by_period(driver_id, start_date, end_date, page, limit)
            .await?;
        if let Some((cached_workdays, total_count)) = cached_workdays {
            return Ok((cached_workdays, total_count));
        }

        let (workdays, total_count) = self
            .workday_database_repository
            .get_workdays_by_period(driver_id, start_date, end_date, page, limit)
            .await?;
        let workdays_transformed: Vec<Workday> = workdays.iter().map(|w| w.to_workday()).collect();

        self.workday_cache_repository
            .set_workdays_by_period(
                driver_id,
                start_date,
                end_date,
                page,
                limit,
                workdays_transformed.clone(),
                total_count,
            )
            .await?;

        Ok((workdays_transformed, total_count))
    }

    async fn create_workday(
        &self,
        driver_id: Uuid,
        create_workday_request: CreateWorkdayRequest,
    ) -> Result<WorkdayRow, WorkdayError> {
        let workday = self
            .workday_database_repository
            .create_workday(driver_id, create_workday_request)
            .await?;

        self.workday_cache_repository
            .delete_workdays_by_month(driver_id, workday.date.month() as i32, workday.date.year())
            .await?;
        self.workday_cache_repository
            .delete_key(driver_id, "workdays:period")
            .await?;

        Ok(workday)
    }

    async fn update_workday(
        &self,
        driver_id: Uuid,
        update_workday_request: UpdateWorkdayRequest,
    ) -> Result<WorkdayRow, WorkdayError> {
        let workday = self
            .workday_database_repository
            .update_workday(driver_id, update_workday_request)
            .await?;

        self.workday_cache_repository
            .delete_workdays_by_month(driver_id, workday.date.month() as i32, workday.date.year())
            .await?;
        self.workday_cache_repository
            .delete_key(driver_id, "workdays:period")
            .await?;

        Ok(workday)
    }

    async fn delete_workday(&self, driver_id: Uuid, date: NaiveDate) -> Result<(), WorkdayError> {
        self.workday_database_repository
            .delete_workday(driver_id, date)
            .await?;

        self.workday_cache_repository
            .delete_workdays_by_month(driver_id, date.month() as i32, date.year())
            .await?;
        self.workday_cache_repository
            .delete_key(driver_id, "workdays:period")
            .await?;

        Ok(())
    }

    async fn get_workdays_garbage(
        &self,
        driver_id: Uuid,
    ) -> Result<Vec<WorkdayGarbageRow>, WorkdayError> {
        self.workday_database_repository
            .get_workdays_garbage(driver_id)
            .await
    }

    async fn create_workday_garbage(
        &self,
        driver_id: Uuid,
        date: NaiveDate,
    ) -> Result<WorkdayGarbageRow, WorkdayError> {
        let scheduled_deletion_date =
            chrono::Utc::now().naive_utc().date() + chrono::Duration::days(30);
        let workday_garbage = self
            .workday_database_repository
            .create_workday_garbage(driver_id, date, scheduled_deletion_date, None)
            .await?;

        self.workday_cache_repository
            .delete_workdays_by_month(driver_id, date.month() as i32, date.year())
            .await?;
        self.workday_cache_repository
            .delete_key(driver_id, "workdays:period")
            .await?;

        Ok(workday_garbage)
    }

    async fn delete_workday_garbage(
        &self,
        driver_id: Uuid,
        date: NaiveDate,
    ) -> Result<(), WorkdayError> {
        self.workday_database_repository
            .delete_workday_garbage(driver_id, date)
            .await?;

        self.workday_cache_repository
            .delete_workdays_by_month(driver_id, date.month() as i32, date.year())
            .await?;
        self.workday_cache_repository
            .delete_key(driver_id, "workdays:period")
            .await?;

        Ok(())
    }

    async fn get_workday_documents(&self, driver_id: Uuid) -> Result<Vec<i32>, WorkdayError> {
        self.workday_database_repository
            .get_workday_documents(driver_id)
            .await
    }

    async fn get_workday_documents_by_year(
        &self,
        driver_id: Uuid,
        year: i32,
    ) -> Result<Vec<i32>, WorkdayError> {
        self.workday_database_repository
            .get_workday_documents_by_year(driver_id, year)
            .await
    }

    async fn get_workday_document_by_month(
        &self,
        driver_id: Uuid,
        month: i32,
        year: i32,
    ) -> Result<Option<bytes::Bytes>, WorkdayError> {
        let workdays = self.get_workdays_by_month(driver_id, month, year).await?;

        let driver = self
            .driver_database_repository
            .get_driver_by_id(driver_id)
            .await
            .map_err(|_| WorkdayError::Internal)?;

        let driver = driver.ok_or(WorkdayError::Internal)?;

        self.document_external_repository
            .get_workday_documents_by_month(
                driver.firstname,
                driver.lastname,
                driver.language,
                month,
                year,
                workdays,
            )
            .await
            .map_err(|_| WorkdayError::Internal)
    }
}
