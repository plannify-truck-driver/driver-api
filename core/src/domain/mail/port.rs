use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{
    domain::{
        common::constants::EnumDriverMailType,
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
        verify_value: String,
        verify_ttl: u64,
    ) -> impl Future<Output = Result<(), MailError>> + Send;
}

pub trait MailDatabaseRepository: Send + Sync {
    fn create_mail(
        &self,
        driver: DriverRow,
        mail_type: EnumDriverMailType,
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

pub trait MailService: Send + Sync {
    fn send_creation_email(
        &self,
        driver: DriverRow,
    ) -> impl Future<Output = Result<(), MailError>> + Send;
}

pub struct MockMailSmtpRepository;

impl MockMailSmtpRepository {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for MockMailSmtpRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl MailSmtpRepository for MockMailSmtpRepository {
    fn send_email(&self, _to: String, _subject: String, _body: String) -> Result<(), MailError> {
        Ok(())
    }

    async fn send_driver_creation_email(
        &self,
        _driver: DriverRow,
        _verify_value: String,
        _verify_ttl: u64,
    ) -> Result<(), MailError> {
        Ok(())
    }
}

pub struct MockMailDatabaseRepository {
    mails: Arc<Mutex<Vec<DriverMailRow>>>,
}

impl MockMailDatabaseRepository {
    pub fn new() -> Self {
        Self {
            mails: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl Default for MockMailDatabaseRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl MailDatabaseRepository for MockMailDatabaseRepository {
    async fn create_mail(
        &self,
        driver: DriverRow,
        mail_type: EnumDriverMailType,
        description: String,
        content: Option<String>,
    ) -> Result<DriverMailRow, MailError> {
        let mail = DriverMailRow {
            pk_driver_mail_id: Uuid::new_v4(),
            fk_driver_id: driver.pk_driver_id,
            fk_employee_id: None,
            fk_mail_type_id: mail_type.as_id(),
            description,
            content,
            email_used: driver.email.clone(),
            status: MailStatus::PENDING,
            created_at: Utc::now(),
            sent_at: None,
        };

        let mut mails = self.mails.lock().unwrap();
        mails.push(mail.clone());

        Ok(mail)
    }

    async fn update_mail_status(
        &self,
        mail_id: Uuid,
        status: MailStatus,
        sent_at: Option<DateTime<Utc>>,
    ) -> Result<DriverMailRow, MailError> {
        let mails = self.mails.lock().unwrap();
        for mail in mails.iter() {
            if mail.pk_driver_mail_id == mail_id {
                let updated_mail = DriverMailRow {
                    pk_driver_mail_id: mail.pk_driver_mail_id,
                    fk_driver_id: mail.fk_driver_id,
                    fk_employee_id: mail.fk_employee_id,
                    fk_mail_type_id: mail.fk_mail_type_id,
                    email_used: mail.email_used.clone(),
                    description: mail.description.clone(),
                    content: mail.content.clone(),
                    status: status.clone(),
                    created_at: mail.created_at,
                    sent_at,
                };
                return Ok(updated_mail);
            }
        }

        Err(MailError::MailNotFound)
    }
}
