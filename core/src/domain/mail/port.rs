use std::{
    collections::HashMap,
    future::Future,
    sync::{Arc, Mutex},
};

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{
    domain::{
        common::constants::EnumDriverMailType,
        driver::entities::DriverRow,
        mail::entities::{
            DriverMail, DriverMailAttachmentRow, DriverMailPreference,
            DriverMailRow, DriverMailType, DriverMailTypeRow, MailStatus,
        },
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

    fn send_driver_deactivation_email(
        &self,
        driver: DriverRow,
    ) -> impl Future<Output = Result<(), MailError>> + Send;

    fn send_driver_reactivation_email(
        &self,
        driver: DriverRow,
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

    fn get_mails(
        &self,
        driver_id: Uuid,
        page: u32,
        limit: u32,
    ) -> impl Future<Output = Result<(Vec<DriverMailRow>, u32), MailError>> + Send;

    fn get_mail_types(
        &self,
    ) -> impl Future<Output = Result<Vec<DriverMailTypeRow>, MailError>> + Send;

    fn get_mail_type_by_id(
        &self,
        mail_type_id: i32,
    ) -> impl Future<Output = Result<DriverMailTypeRow, MailError>> + Send;

    fn get_driver_mail_preferences(
        &self,
        driver_id: Uuid,
    ) -> impl Future<Output = Result<i32, MailError>> + Send;

    fn update_driver_mail_preferences(
        &self,
        driver_id: Uuid,
        mail_preferences: i32,
    ) -> impl Future<Output = Result<i32, MailError>> + Send;

    fn get_mail_attachments(
        &self,
        mail_ids: Vec<Uuid>,
    ) -> impl Future<Output = Result<Vec<DriverMailAttachmentRow>, MailError>> + Send;

    fn get_mail_by_id(
        &self,
        mail_id: Uuid,
    ) -> impl Future<Output = Result<DriverMailRow, MailError>> + Send;

    fn get_mail_attachment_by_id(
        &self,
        attachment_id: Uuid,
    ) -> impl Future<Output = Result<DriverMailAttachmentRow, MailError>> + Send;
}

pub trait MailService: Send + Sync {
    fn send_creation_email(
        &self,
        driver: DriverRow,
    ) -> impl Future<Output = Result<(), MailError>> + Send;

    fn send_email_change_verification(
        &self,
        driver: DriverRow,
    ) -> impl Future<Output = Result<(), MailError>> + Send;

    fn send_deactivation_email(
        &self,
        driver: DriverRow,
    ) -> impl Future<Output = Result<(), MailError>> + Send;

    fn send_reactivation_email(
        &self,
        driver: DriverRow,
    ) -> impl Future<Output = Result<(), MailError>> + Send;

    fn get_mails(
        &self,
        driver_id: Uuid,
        page: u32,
        limit: u32,
    ) -> impl Future<Output = Result<(Vec<DriverMail>, u32), MailError>> + Send;

    fn get_mail_types(&self)
    -> impl Future<Output = Result<Vec<DriverMailType>, MailError>> + Send;

    fn get_mail_preferences(
        &self,
        driver_id: Uuid,
    ) -> impl Future<Output = Result<Vec<DriverMailPreference>, MailError>> + Send;

    fn update_mail_preference(
        &self,
        driver_id: Uuid,
        mail_type_id: i32,
        is_enabled: bool,
    ) -> impl Future<Output = Result<DriverMailPreference, MailError>> + Send;

    fn get_mail(
        &self,
        driver_id: Uuid,
        mail_id: Uuid,
    ) -> impl Future<Output = Result<DriverMail, MailError>> + Send;

    fn download_mail_attachment(
        &self,
        driver_id: Uuid,
        attachment_id: Uuid,
    ) -> impl Future<Output = Result<(bytes::Bytes, String), MailError>> + Send;
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

    async fn send_driver_deactivation_email(&self, _driver: DriverRow) -> Result<(), MailError> {
        Ok(())
    }

    async fn send_driver_reactivation_email(&self, _driver: DriverRow) -> Result<(), MailError> {
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

    async fn get_mails(
        &self,
        _driver_id: Uuid,
        _page: u32,
        _limit: u32,
    ) -> Result<(Vec<DriverMailRow>, u32), MailError> {
        Ok((Vec::new(), 0))
    }

    async fn get_mail_types(&self) -> Result<Vec<DriverMailTypeRow>, MailError> {
        Ok(Vec::new())
    }

    async fn get_mail_type_by_id(
        &self,
        _mail_type_id: i32,
    ) -> Result<DriverMailTypeRow, MailError> {
        Err(MailError::MailTypeNotFound)
    }

    async fn get_driver_mail_preferences(&self, _driver_id: Uuid) -> Result<i32, MailError> {
        Ok(0)
    }

    async fn update_driver_mail_preferences(
        &self,
        _driver_id: Uuid,
        mail_preferences: i32,
    ) -> Result<i32, MailError> {
        Ok(mail_preferences)
    }

    async fn get_mail_attachments(
        &self,
        _mail_ids: Vec<Uuid>,
    ) -> Result<Vec<DriverMailAttachmentRow>, MailError> {
        Ok(Vec::new())
    }

    async fn get_mail_by_id(&self, _mail_id: Uuid) -> Result<DriverMailRow, MailError> {
        Err(MailError::MailNotFound)
    }

    async fn get_mail_attachment_by_id(
        &self,
        _attachment_id: Uuid,
    ) -> Result<DriverMailAttachmentRow, MailError> {
        Err(MailError::MailAttachmentNotFound)
    }
}

// ── Mail cache ────────────────────────────────────────────────────────────────

pub enum MailCacheKeyType {
    MailsList { page: u32, limit: u32 },
    Mail { mail_id: Uuid },
    MailPreferences,
}

impl MailCacheKeyType {
    pub fn as_str(&self) -> String {
        match self {
            MailCacheKeyType::MailsList { page, limit } => format!("list:{}:{}", page, limit),
            MailCacheKeyType::Mail { mail_id } => format!("item:{}", mail_id),
            MailCacheKeyType::MailPreferences => "preferences".to_string(),
        }
    }

    pub fn to_ttl(&self) -> u64 {
        match self {
            MailCacheKeyType::MailsList { .. } => 3600, // 1h
            MailCacheKeyType::Mail { .. } => 3600,      // 1h
            MailCacheKeyType::MailPreferences => 3600,  // 1h
        }
    }
}

pub trait MailCacheRepository: Send + Sync {
    fn generate_redis_key(&self, driver_id: Uuid, suffix: &str) -> String;

    fn get_key_by_type(&self, driver_id: Uuid, key_type: MailCacheKeyType) -> (String, u64) {
        (
            self.generate_redis_key(driver_id, &key_type.as_str()),
            key_type.to_ttl(),
        )
    }

    fn mail_types_cache_key(&self) -> (String, u64) {
        ("mail:types".to_string(), 86400)
    }

    fn get_mails(
        &self,
        driver_id: Uuid,
        page: u32,
        limit: u32,
    ) -> impl Future<Output = Result<Option<(Vec<DriverMail>, u32)>, MailError>> + Send;

    fn set_mails(
        &self,
        driver_id: Uuid,
        page: u32,
        limit: u32,
        mails: Vec<DriverMail>,
        total: u32,
    ) -> impl Future<Output = Result<(), MailError>> + Send;

    fn delete_mails(&self, driver_id: Uuid) -> impl Future<Output = Result<(), MailError>> + Send;

    fn get_mail_types(
        &self,
    ) -> impl Future<Output = Result<Option<Vec<DriverMailType>>, MailError>> + Send;

    fn set_mail_types(
        &self,
        types: Vec<DriverMailType>,
    ) -> impl Future<Output = Result<(), MailError>> + Send;

    fn get_mail_preferences(
        &self,
        driver_id: Uuid,
    ) -> impl Future<Output = Result<Option<Vec<DriverMailPreference>>, MailError>> + Send;

    fn set_mail_preferences(
        &self,
        driver_id: Uuid,
        preferences: Vec<DriverMailPreference>,
    ) -> impl Future<Output = Result<(), MailError>> + Send;

    fn delete_mail_preferences(
        &self,
        driver_id: Uuid,
    ) -> impl Future<Output = Result<(), MailError>> + Send;

    fn get_mail(
        &self,
        driver_id: Uuid,
        mail_id: Uuid,
    ) -> impl Future<Output = Result<Option<DriverMail>, MailError>> + Send;

    fn set_mail(
        &self,
        driver_id: Uuid,
        mail_id: Uuid,
        mail: DriverMail,
    ) -> impl Future<Output = Result<(), MailError>> + Send;
}

#[derive(Clone)]
#[allow(clippy::type_complexity)]
pub struct MockMailCacheRepository {
    mails: Arc<Mutex<HashMap<String, (Vec<DriverMail>, u32)>>>,
    mail: Arc<Mutex<HashMap<String, DriverMail>>>,
    mail_types: Arc<Mutex<Option<Vec<DriverMailType>>>>,
    preferences: Arc<Mutex<HashMap<Uuid, Vec<DriverMailPreference>>>>,
}

impl MockMailCacheRepository {
    pub fn new() -> Self {
        Self {
            mails: Arc::new(Mutex::new(HashMap::new())),
            mail: Arc::new(Mutex::new(HashMap::new())),
            mail_types: Arc::new(Mutex::new(None)),
            preferences: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Default for MockMailCacheRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl MailCacheRepository for MockMailCacheRepository {
    fn generate_redis_key(&self, driver_id: Uuid, suffix: &str) -> String {
        format!("driver:{}:mails:{}", driver_id, suffix)
    }

    async fn get_mails(
        &self,
        driver_id: Uuid,
        page: u32,
        limit: u32,
    ) -> Result<Option<(Vec<DriverMail>, u32)>, MailError> {
        let (key, _) = self.get_key_by_type(driver_id, MailCacheKeyType::MailsList { page, limit });
        Ok(self.mails.lock().unwrap().get(&key).cloned())
    }

    async fn set_mails(
        &self,
        driver_id: Uuid,
        page: u32,
        limit: u32,
        mails: Vec<DriverMail>,
        total: u32,
    ) -> Result<(), MailError> {
        let (key, _) = self.get_key_by_type(driver_id, MailCacheKeyType::MailsList { page, limit });
        self.mails.lock().unwrap().insert(key, (mails, total));
        Ok(())
    }

    async fn delete_mails(&self, driver_id: Uuid) -> Result<(), MailError> {
        let prefix = self.generate_redis_key(driver_id, "list:");
        self.mails
            .lock()
            .unwrap()
            .retain(|k, _| !k.starts_with(&prefix));
        Ok(())
    }

    async fn get_mail_types(&self) -> Result<Option<Vec<DriverMailType>>, MailError> {
        Ok(self.mail_types.lock().unwrap().clone())
    }

    async fn set_mail_types(&self, types: Vec<DriverMailType>) -> Result<(), MailError> {
        *self.mail_types.lock().unwrap() = Some(types);
        Ok(())
    }

    async fn get_mail_preferences(
        &self,
        driver_id: Uuid,
    ) -> Result<Option<Vec<DriverMailPreference>>, MailError> {
        Ok(self.preferences.lock().unwrap().get(&driver_id).cloned())
    }

    async fn set_mail_preferences(
        &self,
        driver_id: Uuid,
        preferences: Vec<DriverMailPreference>,
    ) -> Result<(), MailError> {
        self.preferences
            .lock()
            .unwrap()
            .insert(driver_id, preferences);
        Ok(())
    }

    async fn delete_mail_preferences(&self, driver_id: Uuid) -> Result<(), MailError> {
        self.preferences.lock().unwrap().remove(&driver_id);
        Ok(())
    }

    async fn get_mail(
        &self,
        driver_id: Uuid,
        mail_id: Uuid,
    ) -> Result<Option<DriverMail>, MailError> {
        let key = self.generate_redis_key(driver_id, &format!("item:{}", mail_id));
        Ok(self.mail.lock().unwrap().get(&key).cloned())
    }

    async fn set_mail(
        &self,
        driver_id: Uuid,
        mail_id: Uuid,
        mail: DriverMail,
    ) -> Result<(), MailError> {
        let key = self.generate_redis_key(driver_id, &format!("item:{}", mail_id));
        self.mail.lock().unwrap().insert(key, mail);
        Ok(())
    }
}
