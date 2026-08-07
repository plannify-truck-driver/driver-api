use std::sync::{Arc, Mutex};

use crate::{
    domain::driver_information::entities::{
        DriverInformation, DriverInformationCache, DriverInformationRow,
    },
    infrastructure::driver_information::repositories::error::DriverInformationError,
};

pub trait DriverInformationDatabaseRepository: Send + Sync {
    fn get_driver_informations(
        &self,
    ) -> impl Future<Output = Result<Vec<DriverInformationRow>, DriverInformationError>> + Send;
}

pub trait DriverInformationCacheRepository: Send + Sync {
    fn driver_informations_cache_key(&self) -> (String, u64) {
        ("driver_informations:list".to_string(), 86400)
    }

    fn get_driver_informations(
        &self,
    ) -> impl Future<Output = Result<Option<Vec<DriverInformationCache>>, DriverInformationError>> + Send;

    fn set_driver_informations(
        &self,
        informations: Vec<DriverInformationCache>,
    ) -> impl Future<Output = Result<(), DriverInformationError>> + Send;
}

pub trait DriverInformationService: Send + Sync {
    fn get_driver_informations(
        &self,
    ) -> impl Future<Output = Result<Vec<DriverInformation>, DriverInformationError>> + Send;
}

#[derive(Clone)]
pub struct MockDriverInformationDatabaseRepository {
    informations: Arc<Mutex<Vec<DriverInformationRow>>>,
}

impl MockDriverInformationDatabaseRepository {
    pub fn new() -> Self {
        Self {
            informations: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn with_informations(informations: Vec<DriverInformationRow>) -> Self {
        Self {
            informations: Arc::new(Mutex::new(informations)),
        }
    }
}

impl Default for MockDriverInformationDatabaseRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl DriverInformationDatabaseRepository for MockDriverInformationDatabaseRepository {
    async fn get_driver_informations(
        &self,
    ) -> Result<Vec<DriverInformationRow>, DriverInformationError> {
        Ok(self.informations.lock().unwrap().clone())
    }
}

#[derive(Clone)]
pub struct MockDriverInformationCacheRepository {
    informations: Arc<Mutex<Option<Vec<DriverInformationCache>>>>,
}

impl MockDriverInformationCacheRepository {
    pub fn new() -> Self {
        Self {
            informations: Arc::new(Mutex::new(None)),
        }
    }
}

impl Default for MockDriverInformationCacheRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl DriverInformationCacheRepository for MockDriverInformationCacheRepository {
    async fn get_driver_informations(
        &self,
    ) -> Result<Option<Vec<DriverInformationCache>>, DriverInformationError> {
        Ok(self.informations.lock().unwrap().clone())
    }

    async fn set_driver_informations(
        &self,
        informations: Vec<DriverInformationCache>,
    ) -> Result<(), DriverInformationError> {
        *self.informations.lock().unwrap() = Some(informations);
        Ok(())
    }
}
