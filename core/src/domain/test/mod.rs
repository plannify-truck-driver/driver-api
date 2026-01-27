use crate::{
    Service,
    domain::{
        driver::port::{MockDriverCacheRepository, MockDriverDatabaseRepository},
        health::port::MockHealthRepository,
        mail::port::{MockMailDatabaseRepository, MockMailSmtpRepository},
        update::port::{MockUpdateCacheRepository, MockUpdateDatabaseRepository},
        workday::port::{MockWorkdayCacheRepository, MockWorkdayDatabaseRepository},
    },
};

pub mod workday;

pub type MockService = Service<
    MockHealthRepository,
    MockDriverDatabaseRepository,
    MockDriverCacheRepository,
    MockWorkdayDatabaseRepository,
    MockWorkdayCacheRepository,
    MockMailSmtpRepository,
    MockMailDatabaseRepository,
    MockUpdateDatabaseRepository,
    MockUpdateCacheRepository,
>;

pub fn create_mock_service() -> MockService {
    let health_repository = MockHealthRepository::new();
    let driver_database_repository = MockDriverDatabaseRepository::new();
    let driver_cache_repository = MockDriverCacheRepository::new();
    let workday_database_repository = MockWorkdayDatabaseRepository::new();
    let workday_cache_repository = MockWorkdayCacheRepository::new();
    let mail_smtp_repository = MockMailSmtpRepository::new();
    let mail_database_repository = MockMailDatabaseRepository::new();
    let update_database_repository = MockUpdateDatabaseRepository::new();
    let update_cache_repository = MockUpdateCacheRepository::new();

    MockService::new(
        health_repository,
        driver_database_repository,
        driver_cache_repository,
        workday_database_repository,
        workday_cache_repository,
        mail_smtp_repository,
        mail_database_repository,
        update_database_repository,
        update_cache_repository,
    )
}
