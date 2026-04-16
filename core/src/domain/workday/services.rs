use chrono::{Datelike, NaiveDate};
use uuid::Uuid;

use crate::{
    Service,
    domain::{
        document::port::DocumentExternalRepository,
        driver::port::{DriverCacheRepository, DriverDatabaseRepository},
        health::port::HealthRepository,
        mail::port::{MailDatabaseRepository, MailSmtpRepository},
        storage::port::StorageRepository,
        update::port::{UpdateCacheRepository, UpdateDatabaseRepository},
        workday::{
            entities::{
                CreateWorkdayRequest, UpdateWorkdayRequest, Workday, WorkdayDocumentInformation,
                WorkdayGarbageRow, WorkdayRow,
            },
            port::{WorkdayCacheRepository, WorkdayDatabaseRepository, WorkdayService},
        },
    },
    infrastructure::workday::repositories::error::WorkdayError,
};

impl<H, DD, DC, WD, WC, MS, MD, UD, UC, DE, DS> WorkdayService
    for Service<H, DD, DC, WD, WC, MS, MD, UD, UC, DE, DS>
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
    DS: StorageRepository,
{
    #[tracing::instrument(
        name = "workday_service.get_workday_by_date",
        skip(self),
        fields(
            driver_id = %driver_id,
            date = %date,
        )
    )]
    async fn get_workday_by_date(
        &self,
        driver_id: Uuid,
        date: NaiveDate,
    ) -> Result<Workday, WorkdayError> {
        let workday = self
            .workday_database_repository
            .get_workday_by_date(driver_id, date)
            .await?;

        if let Some(existing_workday) = workday {
            Ok(existing_workday.to_workday())
        } else {
            Err(WorkdayError::WorkdayNotFound)
        }
    }

    #[tracing::instrument(
        name = "workday_service.get_workdays_by_month",
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
    ) -> Result<Vec<Workday>, WorkdayError> {
        let cached_workdays = self
            .workday_cache_repository
            .get_workdays_by_month(driver_id, month, year)
            .await?;
        if let Some(cached_workdays) = cached_workdays {
            tracing::Span::current().record("cache.hit", true);
            return Ok(cached_workdays);
        }

        tracing::Span::current().record("cache.hit", false);

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

    #[tracing::instrument(
        name = "workday_service.get_workdays_by_period",
        skip(self),
        fields(
            driver_id = %driver_id,
            start_date = %start_date,
            end_date = %end_date,
            page = %page,
            limit = %limit,
            cache.hit = tracing::field::Empty,
        )
    )]
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
            tracing::Span::current().record("cache.hit", true);
            return Ok((cached_workdays, total_count));
        }

        tracing::Span::current().record("cache.hit", false);

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

    #[tracing::instrument(
        name = "workday_service.create_workday",
        skip(self),
        fields(
            driver_id = %driver_id,
            date = %create_workday_request.date,
        )
    )]
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
        self.workday_cache_repository
            .delete_documents_by_year(driver_id, workday.date.year())
            .await?;
        self.workday_cache_repository
            .delete_document_years(driver_id)
            .await?;

        Ok(workday)
    }

    #[tracing::instrument(
        name = "workday_service.update_workday",
        skip(self),
        fields(
            driver_id = %driver_id,
            date = %update_workday_request.date,
        )
    )]
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

    #[tracing::instrument(
        name = "workday_service.delete_workday",
        skip(self),
        fields(
            driver_id = %driver_id,
            date = %date,
        )
    )]
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
        self.workday_cache_repository
            .delete_documents_by_year(driver_id, date.year())
            .await?;
        self.workday_cache_repository
            .delete_document_years(driver_id)
            .await?;

        Ok(())
    }

    #[tracing::instrument(
        name = "workday_service.get_workdays_garbage",
        skip(self),
        fields(
            driver_id = %driver_id,
        )
    )]
    async fn get_workdays_garbage(
        &self,
        driver_id: Uuid,
    ) -> Result<Vec<WorkdayGarbageRow>, WorkdayError> {
        self.workday_database_repository
            .get_workdays_garbage(driver_id)
            .await
    }

    #[tracing::instrument(
        name = "workday_service.create_workday_garbage",
        skip(self),
        fields(
            driver_id = %driver_id,
            date = %date,
        )
    )]
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
        self.workday_cache_repository
            .delete_documents_by_year(driver_id, date.year())
            .await?;
        self.workday_cache_repository
            .delete_document_years(driver_id)
            .await?;

        Ok(workday_garbage)
    }

    #[tracing::instrument(
        name = "workday_service.delete_workday_garbage",
        skip(self),
        fields(
            driver_id = %driver_id,
            date = %date,
        )
    )]
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
        self.workday_cache_repository
            .delete_documents_by_year(driver_id, date.year())
            .await?;
        self.workday_cache_repository
            .delete_document_years(driver_id)
            .await?;

        Ok(())
    }

    #[tracing::instrument(
        name = "workday_service.get_workday_documents",
        skip(self),
        fields(
            driver_id = %driver_id,
            cache.hit = tracing::field::Empty,
        )
    )]
    async fn get_workday_documents(&self, driver_id: Uuid) -> Result<Vec<i32>, WorkdayError> {
        let cached_years = self
            .workday_cache_repository
            .get_document_years(driver_id)
            .await?;

        if let Some(cached_years) = cached_years {
            tracing::Span::current().record("cache.hit", true);
            return Ok(cached_years);
        }

        tracing::Span::current().record("cache.hit", false);

        let years = self
            .workday_database_repository
            .get_workday_years(driver_id)
            .await?;

        self.workday_cache_repository
            .set_document_years(driver_id, years.clone())
            .await?;

        Ok(years)
    }

    #[tracing::instrument(
        name = "workday_service.get_workday_documents_by_year",
        skip(self),
        fields(
            driver_id = %driver_id,
            year = %year,
            cache.hit = tracing::field::Empty,
        )
    )]
    async fn get_workday_documents_by_year(
        &self,
        driver_id: Uuid,
        year: i32,
    ) -> Result<Vec<WorkdayDocumentInformation>, WorkdayError> {
        if let Some(cached) = self
            .workday_cache_repository
            .get_documents_by_year(driver_id, year)
            .await?
        {
            tracing::Span::current().record("cache.hit", true);
            return Ok(cached);
        }

        tracing::Span::current().record("cache.hit", false);

        let documents = self
            .workday_database_repository
            .get_workday_documents_by_year(driver_id, year)
            .await?;

        let workday_months = self
            .workday_database_repository
            .get_workday_months_by_year(driver_id, year)
            .await?;

        let document_months: std::collections::HashSet<u32> =
            documents.iter().map(|d| d.month).collect();

        let mut result = documents;
        for month in workday_months {
            if !document_months.contains(&(month as u32)) {
                result.push(WorkdayDocumentInformation {
                    month: month as u32,
                    year: year as u32,
                    generated_at: None,
                });
            }
        }

        result.sort_by_key(|d| d.month);

        self.workday_cache_repository
            .set_documents_by_year(driver_id, year, result.clone())
            .await?;

        Ok(result)
    }

    #[tracing::instrument(
        name = "workday_service.get_workday_document_by_month",
        skip(self),
        fields(
            driver_id = %driver_id,
            month = %month,
            year = %year,
            source = tracing::field::Empty,
        )
    )]
    async fn get_workday_document_by_month(
        &self,
        driver_id: Uuid,
        month: i32,
        year: i32,
    ) -> Result<Option<bytes::Bytes>, WorkdayError> {
        // Check cache:
        //    - None        → cache miss, must query DB
        //    - Some(None)  → cached absence, skip DB entirely
        //    - Some(Some)  → cached record, fetch from S3
        let cached_record = self
            .workday_cache_repository
            .get_workday_document_record(driver_id, month, year)
            .await?;

        let document_record = match cached_record {
            Some(record) => record, // Some(None) = absence, Some(Some(doc)) = hit
            None => {
                let db_record = self
                    .workday_database_repository
                    .get_workday_document_record(driver_id, month, year)
                    .await?;

                let _ = self
                    .workday_cache_repository
                    .set_workday_document_record(driver_id, month, year, db_record.clone())
                    .await;

                db_record
            }
        };

        if let Some(record) = document_record {
            tracing::Span::current().record("source", "s3");

            let pdf = self
                .storage_repository
                .download(&record.s3_file_path)
                .await
                .map_err(|_| WorkdayError::Internal)?;

            return Ok(Some(pdf));
        }

        tracing::Span::current().record("source", "grpc");

        let workdays = self.get_workdays_by_month(driver_id, month, year).await?;

        let driver = self
            .driver_database_repository
            .get_driver_by_id(driver_id)
            .await
            .map_err(|_| WorkdayError::Internal)?
            .ok_or(WorkdayError::Internal)?;

        let pdf_opt = self
            .document_external_repository
            .get_workday_documents_by_month(
                driver.firstname,
                driver.lastname,
                driver.language,
                month,
                year,
                workdays,
            )
            .await
            .map_err(|_| WorkdayError::Internal)?;

        Ok(pdf_opt)
    }
}
