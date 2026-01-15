use sqlx::{PgPool, postgres::PgPoolOptions};

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
    pool: PgPool,
    pub health_repository: PostgresHealthRepository,
    pub driver_repository: PostgresDriverRepository,
    pub workday_repository: PostgresWorkdayRepository,
}

pub async fn create_repositories(database_url: &String) -> Result<DriverRepositories, CoreError> {
    let pg_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
        .map_err(|e| CoreError::ServiceUnavailable(e.to_string()))?;

    let health_repository = PostgresHealthRepository::new(pg_pool.clone());
    let driver_repository = PostgresDriverRepository::new(pg_pool.clone());
    let workday_repository = PostgresWorkdayRepository::new(pg_pool.clone());

    Ok(DriverRepositories {
        pool: pg_pool,
        health_repository,
        driver_repository,
        workday_repository,
    })
}

impl From<DriverRepositories> for DriverService {
    fn from(val: DriverRepositories) -> Self {
        Service::new(
            val.health_repository,
            val.driver_repository,
            val.workday_repository,
        )
    }
}

impl DriverRepositories {
    /// Shutdown the underlying database pool
    pub async fn shutdown_pool(&self) {
        let _ = &self.pool.close().await;
    }
}

impl DriverService {
    /// Shutdown the underlying database pool
    pub async fn shutdown_pool(&self) {
        self.health_repository.pool.close().await;
    }
}
