use crate::infrastructure::health::repositories::error::HealthError;

#[derive(Clone)]
pub struct IsHealthy {
    pub database: bool,
    pub cache: bool,
    pub storage: bool,
}

impl IsHealthy {
    pub fn new(database: bool, cache: bool, storage: bool) -> Self {
        Self {
            database,
            cache,
            storage,
        }
    }

    pub fn to_result(&self) -> Result<Self, HealthError> {
        if !self.database {
            Err(HealthError::DatabaseUnhealthy)
        } else if !self.cache {
            Err(HealthError::CacheUnhealthy)
        } else if !self.storage {
            Err(HealthError::StorageUnhealthy)
        } else {
            Ok(self.clone())
        }
    }
}
