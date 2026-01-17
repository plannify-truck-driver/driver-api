use crate::{
    domain::driver::entities::{CreateDriverRequest, DriverRow, LoginDriverRequest},
    infrastructure::driver::repositories::error::DriverError,
};
use std::{
    future::Future,
    sync::{Arc, Mutex},
};

pub trait DriverRepository: Send + Sync {
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
}

impl MockDriverRepository {
    pub fn new() -> Self {
        Self {
            drivers: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl Default for MockDriverRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl DriverRepository for MockDriverRepository {
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
}
