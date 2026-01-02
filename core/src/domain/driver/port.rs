use crate::{domain::driver::entities::{CreateDriverRequest, DriverRow}, infrastructure::driver::repositories::error::DriverError};
use std::future::Future;

pub trait DriverRepository: Send + Sync {
    fn create_driver(
        &self,
        create_request: CreateDriverRequest,
    ) -> impl Future<Output = Result<DriverRow, DriverError>> + Send;
}

pub trait DriverService: Send + Sync {
    fn create_driver(
        &self,
        create_request: CreateDriverRequest,
        email_list_deny: Vec<String>,
    ) -> impl Future<Output = Result<DriverRow, DriverError>> + Send;
}