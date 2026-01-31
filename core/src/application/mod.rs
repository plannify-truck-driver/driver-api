use std::sync::Arc;

use lettre::{SmtpTransport, message::MessageBuilder};
use redis::{Client, aio::ConnectionManager};
use sqlx::{PgPool, postgres::PgPoolOptions};
use tera::Tera;

use crate::{
    PostgresHealthRepository, Service,
    domain::common::CoreError,
    infrastructure::{
        document::repositories::grpc::GrpcDocumentRepository,
        driver::repositories::{
            postgres::PostgresDriverRepository, redis::RedisDriverCacheRepository,
        },
        employee::repositories::postgres::PostgresEmployeeRepository,
        mail::repositories::{postgres::PostgresMailRepository, smtp::SmtpMailRepository},
        update::repositories::{
            postgres::PostgresUpdateRepository, redis::RedisUpdateCacheRepository,
        },
        workday::repositories::{
            postgres::PostgresWorkdayRepository, redis::RedisWorkdayRepository,
        },
    },
};

use tracing::error;

pub type DriverService = Service<
    PostgresHealthRepository,
    PostgresDriverRepository,
    RedisDriverCacheRepository,
    PostgresWorkdayRepository,
    RedisWorkdayRepository,
    SmtpMailRepository,
    PostgresMailRepository,
    PostgresUpdateRepository,
    RedisUpdateCacheRepository,
    GrpcDocumentRepository,
>;

#[derive(Clone)]
pub struct DriverRepositories {
    pub pool: PgPool,
    pub health_repository: PostgresHealthRepository,
    pub driver_database_repository: PostgresDriverRepository,
    pub driver_cache_repository: RedisDriverCacheRepository,
    pub employee_repository: PostgresEmployeeRepository,
    pub workday_database_repository: PostgresWorkdayRepository,
    pub workday_cache_repository: RedisWorkdayRepository,
    pub mail_smtp_repository: SmtpMailRepository,
    pub mail_database_repository: PostgresMailRepository,
    pub update_database_repository: PostgresUpdateRepository,
    pub update_cache_repository: RedisUpdateCacheRepository,
    pub document_external_repository: GrpcDocumentRepository,
}

pub async fn create_repositories(
    database_url: &str,
    redis_url: &str,
    mail_client: MessageBuilder,
    transport: SmtpTransport,
    frontend_url: String,
    is_test_environment: bool,
    pdf_service_endpoint: &str,
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

    let tera = match Tera::new("core/templates/mails/**/*.html") {
        Ok(t) => t,
        Err(e) => {
            error!("Templating parsing error: {}", e);
            return Err(CoreError::ServiceUnavailable(e.to_string()));
        }
    };

    let health_repository = PostgresHealthRepository::new(pg_pool.clone(), redis_manager.clone());
    let driver_database_repository = PostgresDriverRepository::new(pg_pool.clone());
    let driver_cache_repository = RedisDriverCacheRepository::new(redis_manager.clone());
    let employee_repository = PostgresEmployeeRepository::new(pg_pool.clone());
    let workday_database_repository = PostgresWorkdayRepository::new(pg_pool.clone());
    let workday_cache_repository = RedisWorkdayRepository::new(redis_manager.clone());
    let mail_smtp_repository = SmtpMailRepository::new(
        mail_client,
        transport,
        Arc::new(tera),
        frontend_url,
        is_test_environment,
    );
    let mail_database_repository = PostgresMailRepository::new(pg_pool.clone());
    let update_database_repository = PostgresUpdateRepository::new(pg_pool.clone());
    let update_cache_repository = RedisUpdateCacheRepository::new(redis_manager);

    let document_external_repository = GrpcDocumentRepository::connect(pdf_service_endpoint)
        .await
        .map_err(|e| CoreError::ServiceUnavailable(e.to_string()))?;

    Ok(DriverRepositories {
        pool: pg_pool,
        health_repository,
        driver_database_repository,
        driver_cache_repository,
        employee_repository,
        workday_database_repository,
        workday_cache_repository,
        mail_smtp_repository,
        mail_database_repository,
        update_database_repository,
        update_cache_repository,
        document_external_repository,
    })
}

impl From<DriverRepositories> for DriverService {
    fn from(val: DriverRepositories) -> Self {
        Service::new(
            val.health_repository,
            val.driver_database_repository,
            val.driver_cache_repository,
            val.workday_database_repository,
            val.workday_cache_repository,
            val.mail_smtp_repository,
            val.mail_database_repository,
            val.update_database_repository,
            val.update_cache_repository,
            val.document_external_repository,
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
