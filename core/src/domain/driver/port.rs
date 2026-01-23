use chrono::{DateTime, Utc};
use passwords::PasswordGenerator;
use serde_json::Value;
use uuid::Uuid;

use tracing::error;

use crate::{
    domain::driver::entities::{
        CreateDriverRequest, CreateDriverRestPeriodRequest, DriverLimitationRow, DriverRestPeriod,
        DriverRow, DriverSuspensionRow, LoginDriverRequest,
    },
    infrastructure::driver::repositories::error::DriverError,
};
use std::{
    collections::HashMap,
    future::Future,
    sync::{Arc, Mutex},
};

pub trait DriverRepository: Send + Sync {
    fn get_number_of_drivers(&self) -> impl Future<Output = Result<i64, DriverError>> + Send;

    fn get_driver_by_id(
        &self,
        driver_id: Uuid,
    ) -> impl Future<Output = Result<DriverRow, DriverError>> + Send;

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

    fn get_current_driver_suspension(
        &self,
        driver_id: Uuid,
    ) -> impl Future<Output = Result<Option<DriverSuspensionRow>, DriverError>> + Send;

    fn create_driver_suspension(
        &self,
        suspension: DriverSuspensionRow,
    ) -> impl Future<Output = Result<DriverSuspensionRow, DriverError>> + Send;

    fn delete_driver_suspension(
        &self,
        suspension_id: i32,
    ) -> impl Future<Output = Result<(), DriverError>> + Send;

    fn get_driver_rest_periods(
        &self,
        driver_id: Uuid,
    ) -> impl Future<Output = Result<Vec<DriverRestPeriod>, DriverError>> + Send;

    fn set_driver_rest_periods(
        &self,
        driver_id: Uuid,
        rest_periods: Vec<DriverRestPeriod>,
    ) -> impl Future<Output = Result<(), DriverError>> + Send;

    fn delete_driver_rest_periods(
        &self,
        driver_id: Uuid,
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

    fn get_driver_rest_periods(
        &self,
        driver_id: Uuid,
    ) -> impl Future<Output = Result<Vec<DriverRestPeriod>, DriverError>> + Send;

    fn set_driver_rest_periods(
        &self,
        driver_id: Uuid,
        rest_periods: Vec<CreateDriverRestPeriodRequest>,
    ) -> impl Future<Output = Result<(), DriverError>> + Send;

    fn delete_driver_rest_periods(
        &self,
        driver_id: Uuid,
    ) -> impl Future<Output = Result<(), DriverError>> + Send;
}

#[derive(Clone)]
pub struct MockDriverRepository {
    drivers: Arc<Mutex<Vec<DriverRow>>>,
    limitations: Arc<Mutex<Option<DriverLimitationRow>>>,
    suspensions: Arc<Mutex<Vec<DriverSuspensionRow>>>,
}

impl MockDriverRepository {
    pub fn new() -> Self {
        Self {
            drivers: Arc::new(Mutex::new(Vec::new())),
            limitations: Arc::new(Mutex::new(None)),
            suspensions: Arc::new(Mutex::new(Vec::new())),
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

    async fn get_driver_by_id(&self, driver_id: Uuid) -> Result<DriverRow, DriverError> {
        let drivers = self.drivers.lock().unwrap();
        let driver = drivers.iter().find(|d| d.pk_driver_id == driver_id);
        match driver {
            Some(d) => Ok(d.clone()),
            None => Err(DriverError::DriverNotFound),
        }
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

    async fn get_current_driver_suspension(
        &self,
        driver_id: Uuid,
    ) -> Result<Option<DriverSuspensionRow>, DriverError> {
        let suspensions = self.suspensions.lock().unwrap();
        let suspension = suspensions.iter().find(|s| {
            s.fk_driver_id == driver_id
                && s.start_at <= chrono::Utc::now()
                && (s.end_at.is_none() || s.end_at.unwrap() >= chrono::Utc::now())
        });
        Ok(suspension.cloned())
    }

    async fn create_driver_suspension(
        &self,
        suspension: DriverSuspensionRow,
    ) -> Result<DriverSuspensionRow, DriverError> {
        let mut suspensions = self.suspensions.lock().unwrap();
        suspensions.push(suspension.clone());
        Ok(suspension)
    }

    async fn delete_driver_suspension(&self, suspension_id: i32) -> Result<(), DriverError> {
        let mut suspensions = self.suspensions.lock().unwrap();
        let initial_len = suspensions.len();
        suspensions.retain(|s| s.pk_driver_suspension_id != suspension_id);
        if suspensions.len() == initial_len {
            return Err(DriverError::DriverSuspensionNotFound);
        }

        Ok(())
    }

    async fn get_driver_rest_periods(
        &self,
        driver_id: Uuid,
    ) -> Result<Vec<DriverRestPeriod>, DriverError> {
        let drivers = self.drivers.lock().unwrap();
        let driver = drivers.iter().find(|d| d.pk_driver_id == driver_id);
        match driver {
            Some(d) => {
                if let Some(rest_json) = &d.rest_json {
                    let rest_periods: Vec<DriverRestPeriod> =
                        serde_json::from_value(rest_json.clone())
                            .map_err(|_| DriverError::Internal)?;
                    Ok(rest_periods)
                } else {
                    Ok(Vec::new())
                }
            }
            None => Err(DriverError::DriverNotFound),
        }
    }

    async fn set_driver_rest_periods(
        &self,
        driver_id: Uuid,
        rest_periods: Vec<DriverRestPeriod>,
    ) -> Result<(), DriverError> {
        let mut drivers = self.drivers.lock().unwrap();
        for driver in drivers.iter_mut() {
            if driver.pk_driver_id == driver_id {
                let rest_json =
                    serde_json::to_string(&rest_periods).map_err(|_| DriverError::Internal)?;
                driver.rest_json = Some(Value::String(rest_json));
                return Ok(());
            }
        }
        Err(DriverError::DriverNotFound)
    }

    async fn delete_driver_rest_periods(&self, driver_id: Uuid) -> Result<(), DriverError> {
        let mut drivers = self.drivers.lock().unwrap();
        for driver in drivers.iter_mut() {
            if driver.pk_driver_id == driver_id {
                driver.rest_json = None;
                return Ok(());
            }
        }
        Err(DriverError::DriverNotFound)
    }
}

pub enum DriverCacheKeyType {
    VerifyEmail,
    ResetPassword,
}

impl DriverCacheKeyType {
    pub fn as_str(&self) -> &str {
        match self {
            DriverCacheKeyType::VerifyEmail => "verify_email",
            DriverCacheKeyType::ResetPassword => "reset_password",
        }
    }
}

pub trait DriverCacheRepository: Send + Sync {
    fn generate_random_value(
        &self,
        length: usize,
    ) -> impl Future<Output = Result<String, DriverError>> + Send;

    fn generate_redis_key(&self, driver_id: Uuid, suffix: &str) -> String;

    fn set_redis(
        &self,
        key: String,
        value: String,
        ttl_seconds: u64,
    ) -> impl Future<Output = Result<(), DriverError>> + Send;

    fn get_redis(
        &self,
        key: String,
    ) -> impl Future<Output = Result<Option<String>, DriverError>> + Send;

    fn get_key_by_type(&self, driver_id: Uuid, key_type: DriverCacheKeyType) -> String {
        self.generate_redis_key(driver_id, key_type.as_str())
    }
}

type MockDriverCacheType = HashMap<String, (String, DateTime<Utc>)>;

#[derive(Clone)]
pub struct MockDriverCacheRepository {
    cache: Arc<Mutex<MockDriverCacheType>>,
}

impl MockDriverCacheRepository {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Default for MockDriverCacheRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl DriverCacheRepository for MockDriverCacheRepository {
    async fn generate_random_value(&self, length: usize) -> Result<String, DriverError> {
        let generator = PasswordGenerator {
            length,
            numbers: true,
            lowercase_letters: true,
            uppercase_letters: true,
            symbols: false,
            spaces: false,
            exclude_similar_characters: false,
            strict: true,
        };

        match generator.generate_one() {
            Ok(key) => Ok(key),
            Err(e) => {
                error!("Failed to generate random key: {:?}", e);
                Err(DriverError::Internal)
            }
        }
    }

    fn generate_redis_key(&self, driver_id: Uuid, suffix: &str) -> String {
        format!("driver:{}:{}", driver_id, suffix)
    }

    async fn set_redis(
        &self,
        key: String,
        value: String,
        ttl_seconds: u64,
    ) -> Result<(), DriverError> {
        let mut cache = self.cache.lock().unwrap();
        let expiry = Utc::now() + chrono::Duration::seconds(ttl_seconds as i64);
        cache.insert(key, (value, expiry));
        Ok(())
    }

    async fn get_redis(&self, key: String) -> Result<Option<String>, DriverError> {
        let mut cache = self.cache.lock().unwrap();
        if let Some((value, expiry)) = cache.get(&key) {
            if *expiry > Utc::now() {
                return Ok(Some(value.clone()));
            } else {
                cache.remove(&key);
            }
        }
        Ok(None)
    }
}
