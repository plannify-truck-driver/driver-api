use crate::{
    domain::health::entities::IsHealthy, infrastructure::health::repositories::error::HealthError,
};
use std::future::Future;

pub trait HealthRepository: Send + Sync {
    fn ping(&self) -> impl Future<Output = IsHealthy> + Send;
}

pub trait HealthService: Send + Sync {
    fn check_health(&self) -> impl Future<Output = Result<IsHealthy, HealthError>> + Send;
}
pub struct MockHealthRepository;

impl MockHealthRepository {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MockHealthRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl HealthRepository for MockHealthRepository {
    async fn ping(&self) -> IsHealthy {
        IsHealthy::new(true, true)
    }
}
