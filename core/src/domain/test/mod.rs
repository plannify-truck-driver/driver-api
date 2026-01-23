use crate::{
    Service,
    domain::{
        driver::port::{MockDriverCacheRepository, MockDriverRepository},
        health::port::MockHealthRepository,
        mail::port::MockMailRepository,
        workday::port::MockWorkdayRepository,
    },
};

pub mod workday;

pub type MockService = Service<
    MockHealthRepository,
    MockDriverRepository,
    MockDriverCacheRepository,
    MockWorkdayRepository,
    MockMailRepository,
>;

pub fn create_mock_service() -> MockService {
    let health_repository = MockHealthRepository::new();
    let driver_repository = MockDriverRepository::new();
    let driver_cache_repository = MockDriverCacheRepository::new();
    let workday_repository = MockWorkdayRepository::new();
    let mail_repository = MockMailRepository::new();

    MockService::new(
        health_repository,
        driver_repository,
        driver_cache_repository,
        workday_repository,
        mail_repository,
    )
}
