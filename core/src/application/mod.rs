use sqlx::postgres::{PgConnectOptions, PgPoolOptions};

use crate::{
    PostgresHealthRepository, Service,
    domain::common::CoreError,
    infrastructure::{
        driver::repositories::postgres::PostgresDriverRepository,
        workday::repositories::postgres::PostgresWorkdayRepository,
    },
};

pub type DriverService =
    Service<PostgresHealthRepository, PostgresDriverRepository, PostgresWorkdayRepository>;

#[derive(Clone)]
pub struct DriverRepositories {
    pub health_repository: PostgresHealthRepository,
    pub driver_repository: PostgresDriverRepository,
    pub workday_repository: PostgresWorkdayRepository,
}

pub async fn create_repositories(
    pg_connection_options: PgConnectOptions,
) -> Result<DriverRepositories, CoreError> {
    let pg_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect_with(pg_connection_options)
        .await
        .map_err(|e| CoreError::ServiceUnavailable(e.to_string()))?;

    let health_repository = PostgresHealthRepository::new(pg_pool.clone());
    let driver_repository = PostgresDriverRepository::new(pg_pool.clone());
    let workday_repository = PostgresWorkdayRepository::new(pg_pool.clone());

    Ok(DriverRepositories {
        health_repository,
        driver_repository,
        workday_repository,
    })
}

impl Into<DriverService> for DriverRepositories {
    fn into(self) -> DriverService {
        Service::new(
            self.health_repository,
            self.driver_repository,
            self.workday_repository,
        )
    }
}
