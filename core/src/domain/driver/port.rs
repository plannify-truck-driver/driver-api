use uuid::Uuid;

use crate::{domain::driver::entities::{CreateDriverRequest, DriverRow}, infrastructure::driver::repositories::error::DriverError};
use std::future::Future;

pub trait DriverRepository: Send + Sync {
    fn create_driver(
        &self,
        create_request: CreateDriverRequest,
    ) -> impl Future<Output = Result<DriverRow, DriverError>> + Send;

    fn update_driver(
        &self,
        driver: DriverRow,
    ) -> impl Future<Output = Result<DriverRow, DriverError>> + Send;
}

pub trait DriverService: Send + Sync {
    fn to_title_case(name: String) -> String;

    fn create_driver(
        &self,
        create_request: CreateDriverRequest,
        email_list_deny: Vec<String>,
    ) -> impl Future<Output = Result<DriverRow, DriverError>> + Send;

    fn generate_tokens<F>(
        &self,
        driver: DriverRow,
        create_tokens: F,
        refresh_ttl: u64,
    ) -> impl Future<Output = Result<(String, String), DriverError>> + Send
    where
        F: Fn(Uuid) -> Result<(String, String), DriverError> + Send + Sync;
}