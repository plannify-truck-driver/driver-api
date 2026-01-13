use std::future::Future;

use chrono::NaiveDate;
use uuid::Uuid;

use crate::{
    domain::workday::entities::{CreateWorkdayRequest, UpdateWorkdayRequest, WorkdayRow},
    infrastructure::workday::repositories::error::WorkdayError,
};

pub trait WorkdayRepository: Send + Sync {
    fn get_workdays_by_month(
        &self,
        driver_id: Uuid,
        month: i32,
        year: i32,
    ) -> impl Future<Output = Result<Vec<WorkdayRow>, WorkdayError>> + Send;

    fn get_workdays_by_period(
        &self,
        driver_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
        page: u32,
        limit: u32,
    ) -> impl Future<Output = Result<(Vec<WorkdayRow>, u32), WorkdayError>> + Send;

    fn create_workday(
        &self,
        driver_id: Uuid,
        create_workday_request: CreateWorkdayRequest,
    ) -> impl Future<Output = Result<WorkdayRow, WorkdayError>> + Send;

    fn update_workday(
        &self,
        driver_id: Uuid,
        update_workday_request: UpdateWorkdayRequest,
    ) -> impl Future<Output = Result<WorkdayRow, WorkdayError>> + Send;

    fn delete_workday(
        &self,
        driver_id: Uuid,
        date: NaiveDate,
    ) -> impl Future<Output = Result<(), WorkdayError>> + Send;

    fn get_workday_documents(
        &self,
        driver_id: Uuid,
    ) -> impl Future<Output = Result<Vec<i32>, WorkdayError>> + Send;

    fn get_workday_documents_by_year(
        &self,
        driver_id: Uuid,
        year: i32,
    ) -> impl Future<Output = Result<Vec<i32>, WorkdayError>> + Send;
}

pub trait WorkdayService: Send + Sync {
    fn get_workdays_by_month(
        &self,
        driver_id: Uuid,
        month: i32,
        year: i32,
    ) -> impl Future<Output = Result<Vec<WorkdayRow>, WorkdayError>> + Send;

    fn get_workdays_by_period(
        &self,
        driver_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
        page: u32,
        limit: u32,
    ) -> impl Future<Output = Result<(Vec<WorkdayRow>, u32), WorkdayError>> + Send;

    fn create_workday(
        &self,
        driver_id: Uuid,
        create_workday_request: CreateWorkdayRequest,
    ) -> impl Future<Output = Result<WorkdayRow, WorkdayError>> + Send;

    fn update_workday(
        &self,
        driver_id: Uuid,
        update_workday_request: UpdateWorkdayRequest,
    ) -> impl Future<Output = Result<WorkdayRow, WorkdayError>> + Send;

    fn delete_workday(
        &self,
        driver_id: Uuid,
        date: NaiveDate,
    ) -> impl Future<Output = Result<(), WorkdayError>> + Send;

    fn get_workday_documents(
        &self,
        driver_id: Uuid,
    ) -> impl Future<Output = Result<Vec<i32>, WorkdayError>> + Send;

    fn get_workday_documents_by_year(
        &self,
        driver_id: Uuid,
        year: i32,
    ) -> impl Future<Output = Result<Vec<i32>, WorkdayError>> + Send;
}
