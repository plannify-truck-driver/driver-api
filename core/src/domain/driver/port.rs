use uuid::Uuid;

use crate::{
    domain::driver::entities::{
        CreateDriverRequest, DriverLimitationRow, DriverRow, LoginDriverRequest,
    },
    infrastructure::driver::repositories::error::DriverError,
};
use std::{
    future::Future,
    sync::{Arc, Mutex},
};

pub trait DriverRepository: Send + Sync {
    fn get_number_of_drivers(&self) -> impl Future<Output = Result<i64, DriverError>> + Send;

    fn get_driver_by_email(
        &self,
        email: String,
    ) -> impl Future<Output = Result<DriverRow, DriverError>> + Send;

    fn create_driver(
        &self,
        create_request: CreateDriverRequest,
    ) -> impl Future<Output = Result<DriverRow, DriverError>> + Send;

    fn update_driver(
        &self,
        driver: DriverRow,
    ) -> impl Future<Output = Result<DriverRow, DriverError>> + Send;

    fn delete_driver(
        &self,
        driver_id: Uuid,
    ) -> impl Future<Output = Result<(), DriverError>> + Send;

    fn get_actual_driver_limitation(
        &self,
    ) -> impl Future<Output = Result<Option<DriverLimitationRow>, DriverError>> + Send;

    fn create_driver_limitation(
        &self,
        limitation: DriverLimitationRow,
    ) -> impl Future<Output = Result<DriverLimitationRow, DriverError>> + Send;

    fn delete_driver_limitation(
        &self,
        limitation_id: i32,
    ) -> impl Future<Output = Result<(), DriverError>> + Send;
}

pub trait DriverService: Send + Sync {
    fn to_title_case(name: String) -> String;

    fn create_driver(
        &self,
        create_request: CreateDriverRequest,
        email_list_deny: Vec<String>,
    ) -> impl Future<Output = Result<DriverRow, DriverError>> + Send;

    fn login_driver(
        &self,
        login_request: LoginDriverRequest,
    ) -> impl Future<Output = Result<DriverRow, DriverError>> + Send;

    fn generate_tokens<F>(
        &self,
        driver: DriverRow,
        create_tokens: F,
        refresh_ttl: u64,
    ) -> impl Future<Output = Result<(String, String), DriverError>> + Send
    where
        F: Fn(&DriverRow) -> Result<(String, String), DriverError> + Send + Sync;
}

#[derive(Clone)]
pub struct MockDriverRepository {
    drivers: Arc<Mutex<Vec<DriverRow>>>,
    limitations: Arc<Mutex<Option<DriverLimitationRow>>>,
}

impl MockDriverRepository {
    pub fn new() -> Self {
        Self {
            drivers: Arc::new(Mutex::new(Vec::new())),
            limitations: Arc::new(Mutex::new(None)),
        }
    }
}

impl Default for MockDriverRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl DriverRepository for MockDriverRepository {
    async fn get_number_of_drivers(&self) -> Result<i64, DriverError> {
        let drivers = self.drivers.lock().unwrap();
        Ok(drivers.len() as i64)
    }

    async fn get_driver_by_email(&self, email: String) -> Result<DriverRow, DriverError> {
        let drivers = self.drivers.lock().unwrap();

        let driver = drivers.iter().find(|d| d.email == email);
        match driver {
            Some(d) => Ok(d.clone()),
            None => Err(DriverError::DriverNotFound),
        }
    }

    async fn create_driver(
        &self,
        create_request: CreateDriverRequest,
    ) -> Result<DriverRow, DriverError> {
        let mut drivers = self.drivers.lock().unwrap();

        if drivers.iter().any(|d| d.email == create_request.email) {
            return Err(DriverError::DriverAlreadyExists);
        }

        let new_driver = DriverRow {
            pk_driver_id: uuid::Uuid::new_v4(),
            firstname: create_request.firstname,
            lastname: create_request.lastname,
            gender: create_request.gender,
            email: create_request.email,
            password_hash: create_request.password,
            phone_number: None,
            is_searchable: false,
            allow_request_professional_agreement: false,
            language: create_request.language,
            rest_json: None,
            mail_preferences: 0,
            created_at: chrono::Utc::now(),
            verified_at: None,
            last_login_at: None,
            deactivated_at: None,
            refresh_token_hash: None,
        };

        drivers.push(new_driver.clone());
        Ok(new_driver)
    }

    async fn update_driver(&self, driver: DriverRow) -> Result<DriverRow, DriverError> {
        let mut drivers = self.drivers.lock().unwrap();
        for existing_driver in drivers.iter_mut() {
            if existing_driver.pk_driver_id == driver.pk_driver_id {
                *existing_driver = driver.clone();
                return Ok(driver);
            }
        }

        Err(DriverError::DriverNotFound)
    }

    async fn delete_driver(&self, driver_id: Uuid) -> Result<(), DriverError> {
        let mut drivers = self.drivers.lock().unwrap();
        let initial_len = drivers.len();

        drivers.retain(|d| d.pk_driver_id != driver_id);
        if drivers.len() == initial_len {
            return Err(DriverError::DriverNotFound);
        }

        Ok(())
    }

    async fn get_actual_driver_limitation(
        &self,
    ) -> Result<Option<DriverLimitationRow>, DriverError> {
        let limitations = self.limitations.lock().unwrap();
        Ok(limitations.clone())
    }

    async fn create_driver_limitation(
        &self,
        limitation: DriverLimitationRow,
    ) -> Result<DriverLimitationRow, DriverError> {
        let mut limitations = self.limitations.lock().unwrap();
        *limitations = Some(limitation.clone());
        Ok(limitation)
    }

    async fn delete_driver_limitation(&self, limitation_id: i32) -> Result<(), DriverError> {
        let mut limitations = self.limitations.lock().unwrap();
        if let Some(limitation) = &*limitations
            && limitation.pk_maximum_entity_limit_id == limitation_id
        {
            *limitations = None;
            return Ok(());
        }
        Err(DriverError::DriverLimitationNotFound)
    }
}
