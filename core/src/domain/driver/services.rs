use crate::{Service, domain::{driver::{entities::{CreateDriverRequest, DriverRow}, port::{DriverRepository, DriverService}}, health::port::HealthRepository}, infrastructure::driver::repositories::error::DriverError};
use tracing::error;

impl <H, D> DriverService for Service<H, D>
where
    H: HealthRepository,
    D: DriverRepository,
{
    async fn create_driver(
        &self,
        create_request: CreateDriverRequest,
        email_list_deny: Vec<String>,
    ) -> Result<DriverRow, DriverError> {
        let email_domain = create_request.email.split('@').last().unwrap_or("");

        if email_list_deny.contains(&email_domain.to_string()) {
            error!("Attempt to sign up with denylisted email domain: {}", email_domain);
            return Err(DriverError::EmailDomainDenylisted {
                domain: email_domain.to_string(),
            });
        }

        self.driver_repository
            .create_driver(create_request)
            .await
    }
}