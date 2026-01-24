use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{
    domain::{
        driver::entities::DriverRow,
        mail::entities::{DriverMailRow, MailStatus},
    },
    infrastructure::mail::repositories::error::MailError,
};

pub trait MailSmtpRepository: Send + Sync {
    fn send_email(&self, to: String, subject: String, body: String) -> Result<(), MailError>;

    fn send_driver_creation_email(
        &self,
        driver: DriverRow,
    ) -> impl Future<Output = Result<(), MailError>> + Send;
}

pub trait MailDatabaseRepository: Send + Sync {
    fn create_mail(
        &self,
        driver: DriverRow,
        mail_type_id: i32,
        description: String,
        content: Option<String>,
    ) -> impl Future<Output = Result<DriverMailRow, MailError>> + Send;

    fn update_mail_status(
        &self,
        mail_id: Uuid,
        status: MailStatus,
        sent_at: Option<DateTime<Utc>>,
    ) -> impl Future<Output = Result<DriverMailRow, MailError>> + Send;
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

impl MailSmtpRepository for MockMailRepository {
    fn send_email(&self, _to: String, _subject: String, _body: String) -> Result<(), MailError> {
        Ok(())
    }

    async fn send_driver_creation_email(&self, _driver: DriverRow) -> Result<(), MailError> {
        Ok(())
    }
}
