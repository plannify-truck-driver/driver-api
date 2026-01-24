use std::sync::Arc;

use lettre::{SmtpTransport, message::MessageBuilder};
use redis::{Client, aio::ConnectionManager};
use sqlx::{PgPool, postgres::PgPoolOptions};
use tera::Tera;

use crate::{
    PostgresHealthRepository, Service,
    domain::common::CoreError,
    infrastructure::{
        driver::repositories::{
            postgres::PostgresDriverRepository, redis::RedisDriverCacheRepository,
        },
        employee::repositories::postgres::PostgresEmployeeRepository,
        mail::repositories::{postgres::PostgresMailRepository, smtp::SmtpMailRepository},
        workday::repositories::postgres::PostgresWorkdayRepository,
    },
};

pub type DriverService = Service<
    PostgresHealthRepository,
    PostgresDriverRepository,
    RedisDriverCacheRepository,
    PostgresWorkdayRepository,
    SmtpMailRepository,
    PostgresMailRepository,
>;

#[derive(Clone)]
pub struct DriverRepositories {
    pub pool: PgPool,
    pub health_repository: PostgresHealthRepository,
    pub driver_repository: PostgresDriverRepository,
    pub driver_cache_repository: RedisDriverCacheRepository,
    pub employee_repository: PostgresEmployeeRepository,
    pub workday_repository: PostgresWorkdayRepository,
    pub mail_smtp_repository: SmtpMailRepository,
    pub mail_database_repository: PostgresMailRepository,
}

pub async fn create_repositories(
    database_url: &str,
    redis_url: &str,
    mail_client: MessageBuilder,
    transport: SmtpTransport,
) -> Result<DriverRepositories, CoreError> {
    let pg_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
        .map_err(|e| CoreError::ServiceUnavailable(e.to_string()))?;

    let redis_client =
        Client::open(redis_url).map_err(|e| CoreError::ServiceUnavailable(e.to_string()))?;
    let redis_manager: ConnectionManager = ConnectionManager::new(redis_client)
        .await
        .map_err(|e| CoreError::ServiceUnavailable(e.to_string()))?;

    let tera = match Tera::new("core/src/templates/mails/**/*.html") {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Parsing error(s): {}", e);
            std::process::exit(1);
        }
    };

    let health_repository = PostgresHealthRepository::new(pg_pool.clone());
    let driver_repository = PostgresDriverRepository::new(pg_pool.clone());
    let driver_cache_repository = RedisDriverCacheRepository::new(redis_manager);
    let employee_repository = PostgresEmployeeRepository::new(pg_pool.clone());
    let workday_repository = PostgresWorkdayRepository::new(pg_pool.clone());
    let mail_smtp_repository = SmtpMailRepository::new(mail_client, transport, Arc::new(tera));
    let mail_database_repository = PostgresMailRepository::new(pg_pool.clone());

    Ok(DriverRepositories {
        pool: pg_pool,
        health_repository,
        driver_repository,
        driver_cache_repository,
        employee_repository,
        workday_repository,
        mail_smtp_repository,
        mail_database_repository,
    })
}

impl From<DriverRepositories> for DriverService {
    fn from(val: DriverRepositories) -> Self {
        Service::new(
            val.health_repository,
            val.driver_repository,
            val.driver_cache_repository,
            val.workday_repository,
            val.mail_smtp_repository,
            val.mail_database_repository,
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
