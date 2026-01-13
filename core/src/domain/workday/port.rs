use std::{
    future::Future,
    sync::{Arc, Mutex},
};

use chrono::{Datelike, NaiveDate};
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

#[derive(Clone)]
pub struct MockWorkdayRepository {
    workdays: Arc<Mutex<Vec<WorkdayRow>>>,
}

impl MockWorkdayRepository {
    pub fn new() -> Self {
        Self {
            workdays: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl WorkdayRepository for MockWorkdayRepository {
    async fn get_workdays_by_month(
        &self,
        driver_id: Uuid,
        month: i32,
        year: i32,
    ) -> Result<Vec<WorkdayRow>, WorkdayError> {
        let workdays = self.workdays.lock().unwrap();
        let result: Vec<WorkdayRow> = workdays
            .iter()
            .filter(|w| {
                w.fk_driver_id == driver_id
                    && w.date.month() as i32 == month
                    && w.date.year() == year
            })
            .cloned()
            .collect();
        Ok(result)
    }

    async fn get_workdays_by_period(
        &self,
        driver_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
        page: u32,
        limit: u32,
    ) -> Result<(Vec<WorkdayRow>, u32), WorkdayError> {
        let workdays = self.workdays.lock().unwrap();
        let filtered: Vec<WorkdayRow> = workdays
            .iter()
            .filter(|w| w.fk_driver_id == driver_id && w.date >= start_date && w.date <= end_date)
            .cloned()
            .collect();

        let total_count = filtered.len() as u32;
        let start = ((page - 1) * limit) as usize;
        let end = (start + limit as usize).min(filtered.len());
        let paginated = if start < filtered.len() {
            filtered[start..end].to_vec()
        } else {
            Vec::new()
        };

        Ok((paginated, total_count))
    }

    async fn create_workday(
        &self,
        driver_id: Uuid,
        create_workday_request: CreateWorkdayRequest,
    ) -> Result<WorkdayRow, WorkdayError> {
        let mut workdays = self.workdays.lock().unwrap();

        if workdays
            .iter()
            .any(|w| w.fk_driver_id == driver_id && w.date == create_workday_request.date)
        {
            return Err(WorkdayError::WorkdayAlreadyExists);
        }

        let new_workday = WorkdayRow {
            date: create_workday_request.date,
            start_time: create_workday_request.start_time,
            end_time: create_workday_request.end_time,
            rest_time: create_workday_request.rest_time,
            overnight_rest: create_workday_request.overnight_rest,
            fk_driver_id: driver_id,
        };

        workdays.push(new_workday.clone());
        Ok(new_workday)
    }

    async fn update_workday(
        &self,
        driver_id: Uuid,
        update_workday_request: UpdateWorkdayRequest,
    ) -> Result<WorkdayRow, WorkdayError> {
        let mut workdays = self.workdays.lock().unwrap();
        if let Some(workday) = workdays
            .iter_mut()
            .find(|w| w.fk_driver_id == driver_id && w.date == update_workday_request.date)
        {
            workday.start_time = update_workday_request.start_time;
            workday.end_time = update_workday_request.end_time;
            workday.rest_time = update_workday_request.rest_time;
            workday.overnight_rest = update_workday_request.overnight_rest;
            Ok(workday.clone())
        } else {
            Err(WorkdayError::WorkdayNotFound)
        }
    }

    async fn delete_workday(&self, driver_id: Uuid, date: NaiveDate) -> Result<(), WorkdayError> {
        let mut workdays = self.workdays.lock().unwrap();
        let initial_len = workdays.len();

        workdays.retain(|w| !(w.fk_driver_id == driver_id && w.date == date));

        if workdays.len() < initial_len {
            Ok(())
        } else {
            Err(WorkdayError::WorkdayNotFound)
        }
    }

    async fn get_workday_documents(&self, driver_id: Uuid) -> Result<Vec<i32>, WorkdayError> {
        let workdays = self.workdays.lock().unwrap();
        let documents: Vec<i32> = workdays
            .iter()
            .filter(|w| w.fk_driver_id == driver_id)
            .map(|w| w.date.year() as i32) // Example logic
            .collect();
        Ok(documents)
    }

    async fn get_workday_documents_by_year(
        &self,
        driver_id: Uuid,
        year: i32,
    ) -> Result<Vec<i32>, WorkdayError> {
        let workdays = self.workdays.lock().unwrap();
        let documents: Vec<i32> = workdays
            .iter()
            .filter(|w| w.fk_driver_id == driver_id && w.date.year() == year)
            .map(|w| w.date.month() as i32) // Example logic
            .collect();
        Ok(documents)
    }
}
