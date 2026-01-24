use chrono::NaiveDate;
use uuid::Uuid;

use crate::{
    Service,
    domain::{
        driver::port::{DriverCacheRepository, DriverRepository},
        health::port::HealthRepository,
        mail::port::MailSmtpRepository,
        workday::{
            entities::{CreateWorkdayRequest, UpdateWorkdayRequest, WorkdayGarbageRow, WorkdayRow},
            port::{WorkdayRepository, WorkdayService},
        },
    },
    infrastructure::workday::repositories::error::WorkdayError,
};

impl<H, D, DC, W, E> WorkdayService for Service<H, D, DC, W, E>
where
    H: HealthRepository,
    D: DriverRepository,
    DC: DriverCacheRepository,
    W: WorkdayRepository,
    E: MailSmtpRepository,
{
    async fn get_workdays_by_month(
        &self,
        driver_id: Uuid,
        month: i32,
        year: i32,
    ) -> Result<Vec<WorkdayRow>, WorkdayError> {
        self.workday_repository
            .get_workdays_by_month(driver_id, month, year)
            .await
    }

    async fn get_workdays_by_period(
        &self,
        driver_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
        page: u32,
        limit: u32,
    ) -> Result<(Vec<WorkdayRow>, u32), WorkdayError> {
        self.workday_repository
            .get_workdays_by_period(driver_id, start_date, end_date, page, limit)
            .await
    }

    async fn create_workday(
        &self,
        driver_id: Uuid,
        create_workday_request: CreateWorkdayRequest,
    ) -> Result<WorkdayRow, WorkdayError> {
        self.workday_repository
            .create_workday(driver_id, create_workday_request)
            .await
    }

    async fn update_workday(
        &self,
        driver_id: Uuid,
        update_workday_request: UpdateWorkdayRequest,
    ) -> Result<WorkdayRow, WorkdayError> {
        self.workday_repository
            .update_workday(driver_id, update_workday_request)
            .await
    }

    async fn delete_workday(&self, driver_id: Uuid, date: NaiveDate) -> Result<(), WorkdayError> {
        self.workday_repository
            .delete_workday(driver_id, date)
            .await
    }

    async fn get_workdays_garbage(
        &self,
        driver_id: Uuid,
    ) -> Result<Vec<WorkdayGarbageRow>, WorkdayError> {
        self.workday_repository
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
        self.workday_repository
            .create_workday_garbage(driver_id, date, scheduled_deletion_date, None)
            .await
    }

    async fn delete_workday_garbage(
        &self,
        driver_id: Uuid,
        date: NaiveDate,
    ) -> Result<(), WorkdayError> {
        self.workday_repository
            .delete_workday_garbage(driver_id, date)
            .await
    }

    async fn get_workday_documents(&self, driver_id: Uuid) -> Result<Vec<i32>, WorkdayError> {
        self.workday_repository
            .get_workday_documents(driver_id)
            .await
    }

    async fn get_workday_documents_by_year(
        &self,
        driver_id: Uuid,
        year: i32,
    ) -> Result<Vec<i32>, WorkdayError> {
        self.workday_repository
            .get_workday_documents_by_year(driver_id, year)
            .await
    }
}
