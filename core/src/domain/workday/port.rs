use std::{
    collections::HashMap,
    future::Future,
    sync::{Arc, Mutex},
};

use chrono::{DateTime, Datelike, NaiveDate, Utc};
use uuid::Uuid;

use crate::{
    domain::workday::entities::{
        CreateWorkdayRequest, UpdateWorkdayRequest, Workday, WorkdayGarbageRow, WorkdayRow,
    },
    infrastructure::workday::repositories::error::WorkdayError,
};

pub enum WorkdayCacheKeyType {
    Monthly { month: i32, year: i32 },
}

impl WorkdayCacheKeyType {
    pub fn as_str(&self) -> String {
        match self {
            WorkdayCacheKeyType::Monthly { month, year } => format!("monthly:{}-{}", month, year),
        }
    }

    pub fn to_ttl(&self) -> u64 {
        match self {
            WorkdayCacheKeyType::Monthly { month: _, year: _ } => 3600 * 6,
        }
    }
}

pub trait WorkdayDatabaseRepository: Send + Sync {
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

    fn get_workdays_garbage(
        &self,
        driver_id: Uuid,
    ) -> impl Future<Output = Result<Vec<WorkdayGarbageRow>, WorkdayError>> + Send;

    fn create_workday_garbage(
        &self,
        driver_id: Uuid,
        date: NaiveDate,
        scheduled_deletion_date: NaiveDate,
        created_at: Option<DateTime<Utc>>,
    ) -> impl Future<Output = Result<WorkdayGarbageRow, WorkdayError>> + Send;

    fn delete_workday_garbage(
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

pub trait WorkdayCacheRepository: Send + Sync {
    fn generate_redis_key(&self, driver_id: Uuid, suffix: &str) -> String;

    fn get_key_by_type(&self, driver_id: Uuid, key_type: WorkdayCacheKeyType) -> (String, u64) {
        (
            self.generate_redis_key(driver_id, &key_type.as_str()),
            key_type.to_ttl(),
        )
    }

    fn get_workdays_by_month(
        &self,
        driver_id: Uuid,
        month: i32,
        year: i32,
    ) -> impl Future<Output = Result<Option<Vec<Workday>>, WorkdayError>> + Send;

    fn set_workdays_by_month(
        &self,
        driver_id: Uuid,
        month: i32,
        year: i32,
        workdays: Vec<Workday>,
    ) -> impl Future<Output = Result<(), WorkdayError>> + Send;

    fn delete_workdays_by_month(
        &self,
        driver_id: Uuid,
        month: i32,
        year: i32,
    ) -> impl Future<Output = Result<(), WorkdayError>> + Send;
}
pub trait WorkdayService: Send + Sync {
    fn get_workdays_by_month(
        &self,
        driver_id: Uuid,
        month: i32,
        year: i32,
    ) -> impl Future<Output = Result<Vec<Workday>, WorkdayError>> + Send;

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

    fn get_workdays_garbage(
        &self,
        driver_id: Uuid,
    ) -> impl Future<Output = Result<Vec<WorkdayGarbageRow>, WorkdayError>> + Send;

    fn create_workday_garbage(
        &self,
        driver_id: Uuid,
        date: NaiveDate,
    ) -> impl Future<Output = Result<WorkdayGarbageRow, WorkdayError>> + Send;

    fn delete_workday_garbage(
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
pub struct MockWorkdayDatabaseRepository {
    workdays: Arc<Mutex<Vec<WorkdayRow>>>,
    workdays_garbage: Arc<Mutex<Vec<WorkdayGarbageRow>>>,
}

impl MockWorkdayDatabaseRepository {
    pub fn new() -> Self {
        Self {
            workdays: Arc::new(Mutex::new(Vec::new())),
            workdays_garbage: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl Default for MockWorkdayDatabaseRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkdayDatabaseRepository for MockWorkdayDatabaseRepository {
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

    async fn get_workdays_garbage(
        &self,
        driver_id: Uuid,
    ) -> Result<Vec<WorkdayGarbageRow>, WorkdayError> {
        let workdays_garbage = self.workdays_garbage.lock().unwrap();
        let result: Vec<WorkdayGarbageRow> = workdays_garbage
            .iter()
            .filter(|w| w.fk_driver_id == driver_id)
            .cloned()
            .collect();
        Ok(result)
    }

    async fn create_workday_garbage(
        &self,
        driver_id: Uuid,
        date: NaiveDate,
        scheduled_deletion_date: NaiveDate,
        created_at: Option<DateTime<Utc>>,
    ) -> Result<WorkdayGarbageRow, WorkdayError> {
        let workdays = self.workdays.lock().unwrap();
        let mut workdays_garbage = self.workdays_garbage.lock().unwrap();

        if !workdays
            .iter()
            .any(|w| w.fk_driver_id == driver_id && w.date == date)
        {
            return Err(WorkdayError::WorkdayNotFound);
        }

        if workdays_garbage
            .iter()
            .any(|w| w.fk_driver_id == driver_id && w.workday_date == date)
        {
            return Err(WorkdayError::WorkdayGarbageAlreadyExists);
        }

        let new_garbage = WorkdayGarbageRow {
            workday_date: date,
            fk_driver_id: driver_id,
            created_at: created_at.unwrap_or_else(Utc::now),
            scheduled_deletion_date,
        };
        workdays_garbage.push(new_garbage.clone());
        Ok(new_garbage)
    }

    async fn delete_workday_garbage(
        &self,
        driver_id: Uuid,
        date: NaiveDate,
    ) -> Result<(), WorkdayError> {
        let mut workdays_garbage = self.workdays_garbage.lock().unwrap();
        let initial_len = workdays_garbage.len();

        workdays_garbage.retain(|w| !(w.fk_driver_id == driver_id && w.workday_date == date));

        if workdays_garbage.len() < initial_len {
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
            .map(|w| w.date.year()) // Example logic
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

type MockWorkdayCacheType = HashMap<String, Vec<Workday>>;

#[derive(Clone)]
pub struct MockWorkdayCacheRepository {
    workdays: Arc<Mutex<MockWorkdayCacheType>>,
}

impl MockWorkdayCacheRepository {
    pub fn new() -> Self {
        Self {
            workdays: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Default for MockWorkdayCacheRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkdayCacheRepository for MockWorkdayCacheRepository {
    fn generate_redis_key(&self, driver_id: Uuid, suffix: &str) -> String {
        format!("driver:{}:workdays:{}", driver_id, suffix)
    }

    async fn get_workdays_by_month(
        &self,
        driver_id: Uuid,
        month: i32,
        year: i32,
    ) -> Result<Option<Vec<Workday>>, WorkdayError> {
        let workdays = self.workdays.lock().unwrap();
        let (key, _) =
            self.get_key_by_type(driver_id, WorkdayCacheKeyType::Monthly { month, year });
        if !workdays.contains_key(&key) {
            return Ok(None);
        }
        let result = workdays.get(&key).cloned().unwrap_or_default();
        Ok(Some(result))
    }

    async fn set_workdays_by_month(
        &self,
        driver_id: Uuid,
        month: i32,
        year: i32,
        workdays: Vec<Workday>,
    ) -> Result<(), WorkdayError> {
        let mut stored_workdays = self.workdays.lock().unwrap();
        let (key, _) =
            self.get_key_by_type(driver_id, WorkdayCacheKeyType::Monthly { month, year });

        stored_workdays.insert(key, workdays);
        Ok(())
    }

    async fn delete_workdays_by_month(
        &self,
        driver_id: Uuid,
        month: i32,
        year: i32,
    ) -> Result<(), WorkdayError> {
        let mut stored_workdays = self.workdays.lock().unwrap();
        let (key, _) =
            self.get_key_by_type(driver_id, WorkdayCacheKeyType::Monthly { month, year });

        stored_workdays.remove(&key);
        Ok(())
    }
}
