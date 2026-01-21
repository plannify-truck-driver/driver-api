use crate::{
    Service,
    domain::{
        driver::port::MockDriverRepository, health::port::MockHealthRepository,
        mail::port::MockMailRepository, workday::port::MockWorkdayRepository,
    },
};

pub mod workday;

pub type MockService =
    Service<MockHealthRepository, MockDriverRepository, MockWorkdayRepository, MockMailRepository>;

pub fn create_mock_service() -> MockService {
    let driver_repository = MockDriverRepository::new();
    let health_repository = MockHealthRepository::new();
    let workday_repository = MockWorkdayRepository::new();
    let mail_repository = MockMailRepository::new();

    MockService::new(
        health_repository,
        driver_repository,
        workday_repository,
        mail_repository,
    )
}
