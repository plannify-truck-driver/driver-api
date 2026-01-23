use crate::{
    domain::driver::entities::DriverRow, infrastructure::mail::repositories::error::MailError,
};

pub trait MailRepository: Send + Sync {
    fn send_email(&self, to: String, subject: String, body: String) -> Result<(), MailError>;

    fn send_driver_creation_email(
        &self,
        driver: DriverRow,
    ) -> impl Future<Output = Result<(), MailError>> + Send;
}

pub struct MockMailRepository;

impl MockMailRepository {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for MockMailRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl MailRepository for MockMailRepository {
    fn send_email(&self, _to: String, _subject: String, _body: String) -> Result<(), MailError> {
        Ok(())
    }

    async fn send_driver_creation_email(&self, _driver: DriverRow) -> Result<(), MailError> {
        Ok(())
    }
}
