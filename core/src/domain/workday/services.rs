use chrono::NaiveDate;
use uuid::Uuid;

use crate::{
    Service,
    domain::{
        driver::port::DriverRepository,
        health::port::HealthRepository,
        workday::{
            entities::{CreateWorkdayRequest, UpdateWorkdayRequest, WorkdayRow},
            port::{WorkdayRepository, WorkdayService},
        },
    },
    infrastructure::workday::repositories::error::WorkdayError,
};

impl<H, D, W> WorkdayService for Service<H, D, W>
where
    H: HealthRepository,
    D: DriverRepository,
    W: WorkdayRepository,
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
}
