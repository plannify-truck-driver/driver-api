use crate::{
    Service,
    domain::{
        driver::port::MockDriverRepository, health::port::MockHealthRepository,
        workday::port::MockWorkdayRepository,
    },
};

pub mod workday;

pub type MockService = Service<MockHealthRepository, MockDriverRepository, MockWorkdayRepository>;

pub fn create_mock_service() -> MockService {
    let driver_repository = MockDriverRepository::new();
    let health_repository = MockHealthRepository::new();
    let workday_repository = MockWorkdayRepository::new();

    MockService::new(health_repository, driver_repository, workday_repository)
}
