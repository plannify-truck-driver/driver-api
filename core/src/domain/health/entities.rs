use crate::infrastructure::health::repositories::error::HealthError;

#[derive(Clone)]
pub struct IsHealthy {
    pub database: bool,
    pub cache: bool,
}

impl IsHealthy {
    pub fn new(database: bool, cache: bool) -> Self {
        Self { database, cache }
    }

    pub fn to_result(&self) -> Result<Self, HealthError> {
        if !self.database {
            Err(HealthError::DatabaseUnhealthy)
        } else if !self.cache {
            Err(HealthError::CacheUnhealthy)
        } else {
            Ok(self.clone())
        }
    }
}
