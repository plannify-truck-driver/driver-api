use plannify_driver_api_core::application::{DriverRepositories, DriverService};

use crate::{AuthValidator, config::Config};

/// Application state shared across request handlers
#[derive(Clone)]
pub struct AppState {
    pub service: DriverService,
    pub config: Config,
    pub auth_validator: AuthValidator,
}

impl AppState {
    /// Create a new AppState with the given service
    pub fn new(service: DriverService, config: Config, auth_validator: AuthValidator) -> Self {
        Self {
            service,
            config,
            auth_validator,
        }
    }

    /// Shutdown the underlying database pool
    pub async fn shutdown(&self) {
        self.service.shutdown_pool().await
    }
}

impl From<DriverRepositories> for AppState {
    fn from(repositories: DriverRepositories) -> Self {
        let service = DriverService::new(
            repositories.health_repository,
            repositories.driver_repository,
            repositories.driver_cache_repository,
            repositories.workday_repository,
            repositories.mail_smtp_repository,
            repositories.mail_database_repository,
            repositories.update_database_repository,
            repositories.update_cache_repository,
        );
        let config = Config::default();
        let jwt_config = &config.jwt;
        let auth_validator = AuthValidator::new(jwt_config);

        AppState {
            service,
            config: Config::default(),
            auth_validator,
        }
    }
}
