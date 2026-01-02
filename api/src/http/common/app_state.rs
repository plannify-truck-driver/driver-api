use plannify_driver_api_core::application::{DriverRepositories, DriverService};

use crate::config::Config;

/// Application state shared across request handlers
#[derive(Clone)]
pub struct AppState {
    pub service: DriverService,
    pub config: Config,
}

impl AppState {
    /// Create a new AppState with the given service
    pub fn new(service: DriverService, config: Config) -> Self {
        Self { service, config }
    }
}

impl From<DriverRepositories> for AppState {
    fn from(repositories: DriverRepositories) -> Self {
        let service = DriverService::new(repositories.health_repository, repositories.driver_repository);
        AppState { service, config: Config::default() }
    }
}
