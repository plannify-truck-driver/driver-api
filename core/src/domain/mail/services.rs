use std::collections::HashMap;
use uuid::Uuid;

use chrono::Utc;

use crate::{
    Service,
    domain::{
        common::constants::EnumDriverMailType,
        document::port::DocumentExternalRepository,
        driver::{
            entities::DriverRow,
            port::{DriverCacheKeyType, DriverCacheRepository, DriverDatabaseRepository},
        },
        health::port::HealthRepository,
        mail::{
            entities::{DriverMail, DriverMailPreference, DriverMailType, MailStatus},
            port::{MailCacheRepository, MailDatabaseRepository, MailService, MailSmtpRepository},
        },
        storage::port::StorageRepository,
        update::port::{UpdateCacheRepository, UpdateDatabaseRepository},
        workday::port::{WorkdayCacheRepository, WorkdayDatabaseRepository},
    },
    infrastructure::{
        mail::repositories::error::MailError, storage::repositories::error::StorageError,
    },
};

impl<H, DD, DC, WD, WC, MS, MD, MC, UD, UC, DE, DS> MailService
    for Service<H, DD, DC, WD, WC, MS, MD, MC, UD, UC, DE, DS>
where
    H: HealthRepository,
    DD: DriverDatabaseRepository,
    DC: DriverCacheRepository,
    WD: WorkdayDatabaseRepository,
    WC: WorkdayCacheRepository,
    MS: MailSmtpRepository,
    MD: MailDatabaseRepository,
    MC: MailCacheRepository,
    UD: UpdateDatabaseRepository,
    UC: UpdateCacheRepository,
    DE: DocumentExternalRepository,
    DS: StorageRepository,
{
    #[tracing::instrument(
        name = "mail_service.send_creation_email",
        skip(self),
        fields(
            driver_id = %driver.pk_driver_id,
        )
    )]
    async fn send_creation_email(&self, driver: DriverRow) -> Result<(), MailError> {
        let bitmask = self
            .mail_database_repository
            .get_driver_mail_preferences(driver.pk_driver_id)
            .await?;

        let bit = 1 << (EnumDriverMailType::AccountVerification.as_id() - 1);
        if bitmask & bit == 0 {
            return Err(MailError::MailPreferenceDisabled);
        }

        let verify_value = self
            .driver_cache_repository
            .generate_random_value(100)
            .await
            .map_err(|_| MailError::Internal)?;

        let (redis_key, redis_ttl) = self
            .driver_cache_repository
            .get_key_by_type(driver.pk_driver_id, DriverCacheKeyType::VerifyEmail);
        let _ = self
            .driver_cache_repository
            .set_redis(redis_key, verify_value.clone(), redis_ttl)
            .await;

        let mail = self
            .mail_database_repository
            .create_mail(
                driver.clone(),
                EnumDriverMailType::AccountVerification,
                "Driver account creation".to_string(),
                None,
            )
            .await?;

        match self
            .mail_smtp_repository
            .send_driver_creation_email(driver.clone(), verify_value, redis_ttl)
            .await
        {
            Ok(_) => {
                self.mail_database_repository
                    .update_mail_status(
                        mail.pk_driver_mail_id,
                        MailStatus::SUCCESS,
                        Some(Utc::now()),
                    )
                    .await?;
            }
            Err(_) => {
                let _ = self
                    .mail_database_repository
                    .update_mail_status(mail.pk_driver_mail_id, MailStatus::FAILED, None)
                    .await?;

                return Err(MailError::Internal);
            }
        }

        Ok(())
    }

    #[tracing::instrument(
        name = "mail_service.send_deactivation_email",
        skip(self),
        fields(driver_id = %driver.pk_driver_id)
    )]
    async fn send_deactivation_email(&self, driver: DriverRow) -> Result<(), MailError> {
        let mail = self
            .mail_database_repository
            .create_mail(
                driver.clone(),
                EnumDriverMailType::AccountChangement,
                "Driver account deactivation".to_string(),
                None,
            )
            .await?;

        match self
            .mail_smtp_repository
            .send_driver_deactivation_email(driver.clone())
            .await
        {
            Ok(_) => {
                self.mail_database_repository
                    .update_mail_status(
                        mail.pk_driver_mail_id,
                        MailStatus::SUCCESS,
                        Some(Utc::now()),
                    )
                    .await?;
            }
            Err(_) => {
                let _ = self
                    .mail_database_repository
                    .update_mail_status(mail.pk_driver_mail_id, MailStatus::FAILED, None)
                    .await?;
                return Err(MailError::Internal);
            }
        }

        Ok(())
    }

    #[tracing::instrument(
        name = "mail_service.send_reactivation_email",
        skip(self),
        fields(driver_id = %driver.pk_driver_id)
    )]
    async fn send_reactivation_email(&self, driver: DriverRow) -> Result<(), MailError> {
        let mail = self
            .mail_database_repository
            .create_mail(
                driver.clone(),
                EnumDriverMailType::AccountChangement,
                "Driver account reactivation".to_string(),
                None,
            )
            .await?;

        match self
            .mail_smtp_repository
            .send_driver_reactivation_email(driver.clone())
            .await
        {
            Ok(_) => {
                self.mail_database_repository
                    .update_mail_status(
                        mail.pk_driver_mail_id,
                        MailStatus::SUCCESS,
                        Some(Utc::now()),
                    )
                    .await?;
            }
            Err(_) => {
                let _ = self
                    .mail_database_repository
                    .update_mail_status(mail.pk_driver_mail_id, MailStatus::FAILED, None)
                    .await?;
                return Err(MailError::Internal);
            }
        }

        Ok(())
    }

    #[tracing::instrument(
        name = "mail_service.get_mails",
        skip(self),
        fields(driver_id = %driver_id, page = %page, limit = %limit)
    )]
    async fn get_mails(
        &self,
        driver_id: Uuid,
        page: u32,
        limit: u32,
    ) -> Result<(Vec<DriverMail>, u32), MailError> {
        if let Some(cached) = self
            .mail_cache_repository
            .get_mails(driver_id, page, limit)
            .await?
        {
            return Ok(cached);
        }

        let (mail_rows, total) = self
            .mail_database_repository
            .get_mails(driver_id, page, limit)
            .await?;

        let mail_ids: Vec<Uuid> = mail_rows.iter().map(|r| r.pk_driver_mail_id).collect();

        let type_rows = self.mail_database_repository.get_mail_types().await?;
        let types_map: HashMap<i32, DriverMailType> = type_rows
            .iter()
            .map(|t| (t.pk_driver_mail_type_id, t.to_driver_mail_type()))
            .collect();

        let attachment_rows = self
            .mail_database_repository
            .get_mail_attachments(mail_ids)
            .await?;

        let mut attachments_by_mail: HashMap<Uuid, Vec<_>> = HashMap::new();
        for row in &attachment_rows {
            attachments_by_mail
                .entry(row.fk_driver_mail_id)
                .or_default()
                .push(row.to_driver_mail_attachment());
        }

        let mails: Vec<DriverMail> = mail_rows
            .iter()
            .filter_map(|row| {
                types_map.get(&row.fk_mail_type_id).map(|t| {
                    let attachments = attachments_by_mail
                        .remove(&row.pk_driver_mail_id)
                        .unwrap_or_default();
                    row.to_driver_mail(t.clone(), attachments)
                })
            })
            .collect();

        let _ = self
            .mail_cache_repository
            .set_mails(driver_id, page, limit, mails.clone(), total)
            .await;

        Ok((mails, total))
    }

    #[tracing::instrument(name = "mail_service.get_mail_types", skip(self))]
    async fn get_mail_types(&self) -> Result<Vec<DriverMailType>, MailError> {
        if let Some(cached) = self.mail_cache_repository.get_mail_types().await? {
            return Ok(cached);
        }

        let rows = self.mail_database_repository.get_mail_types().await?;
        let types: Vec<DriverMailType> = rows.iter().map(|r| r.to_driver_mail_type()).collect();

        let _ = self
            .mail_cache_repository
            .set_mail_types(types.clone())
            .await;

        Ok(types)
    }

    #[tracing::instrument(
        name = "mail_service.get_mail_preferences",
        skip(self),
        fields(driver_id = %driver_id)
    )]
    async fn get_mail_preferences(
        &self,
        driver_id: Uuid,
    ) -> Result<Vec<DriverMailPreference>, MailError> {
        if let Some(cached) = self
            .mail_cache_repository
            .get_mail_preferences(driver_id)
            .await?
        {
            return Ok(cached);
        }

        let bitmask = self
            .mail_database_repository
            .get_driver_mail_preferences(driver_id)
            .await?;

        let type_rows = self.mail_database_repository.get_mail_types().await?;
        let preferences: Vec<DriverMailPreference> = type_rows
            .iter()
            .map(|t| t.to_mail_preference(bitmask))
            .collect();

        let _ = self
            .mail_cache_repository
            .set_mail_preferences(driver_id, preferences.clone())
            .await;

        Ok(preferences)
    }

    #[tracing::instrument(
        name = "mail_service.update_mail_preference",
        skip(self),
        fields(driver_id = %driver_id, mail_type_id = %mail_type_id, is_enabled = %is_enabled)
    )]
    async fn update_mail_preference(
        &self,
        driver_id: Uuid,
        mail_type_id: i32,
        is_enabled: bool,
    ) -> Result<DriverMailPreference, MailError> {
        let mail_type = self
            .mail_database_repository
            .get_mail_type_by_id(mail_type_id)
            .await?;

        if !mail_type.is_editable {
            return Err(MailError::MailPreferenceNotEditable);
        }

        let current = self
            .mail_database_repository
            .get_driver_mail_preferences(driver_id)
            .await?;

        let bit = 1 << (mail_type_id - 1);
        let new_bitmask = if is_enabled {
            current | bit
        } else {
            current & !bit
        };

        let saved = self
            .mail_database_repository
            .update_driver_mail_preferences(driver_id, new_bitmask)
            .await?;

        let _ = self
            .mail_cache_repository
            .delete_mail_preferences(driver_id)
            .await;

        Ok(mail_type.to_mail_preference(saved))
    }

    #[tracing::instrument(
        name = "mail_service.get_mail",
        skip(self),
        fields(driver_id = %driver_id, mail_id = %mail_id)
    )]
    async fn get_mail(&self, driver_id: Uuid, mail_id: Uuid) -> Result<DriverMail, MailError> {
        if let Some(cached) = self
            .mail_cache_repository
            .get_mail(driver_id, mail_id)
            .await?
        {
            return Ok(cached);
        }

        let row = self
            .mail_database_repository
            .get_mail_by_id(mail_id)
            .await?;

        if row.fk_driver_id != driver_id {
            return Err(MailError::MailNotFound);
        }

        let mail_type = self
            .mail_database_repository
            .get_mail_type_by_id(row.fk_mail_type_id)
            .await?
            .to_driver_mail_type();

        let attachment_rows = self
            .mail_database_repository
            .get_mail_attachments(vec![mail_id])
            .await?;

        let attachments = attachment_rows
            .iter()
            .map(|a| a.to_driver_mail_attachment())
            .collect();

        let mail = row.to_driver_mail(mail_type, attachments);

        let _ = self
            .mail_cache_repository
            .set_mail(driver_id, mail_id, mail.clone())
            .await;

        Ok(mail)
    }

    #[tracing::instrument(
        name = "mail_service.get_mail_attachment",
        skip(self),
        fields(driver_id = %driver_id, attachment_id = %attachment_id)
    )]
    async fn download_mail_attachment(
        &self,
        driver_id: Uuid,
        attachment_id: Uuid,
    ) -> Result<(bytes::Bytes, String), MailError> {
        let attachment_row = self
            .mail_database_repository
            .get_mail_attachment_by_id(attachment_id)
            .await?;

        let mail_row = self
            .mail_database_repository
            .get_mail_by_id(attachment_row.fk_driver_mail_id)
            .await?;

        if mail_row.fk_driver_id != driver_id {
            return Err(MailError::MailAttachmentNotFound);
        }

        let bytes = self
            .storage_repository
            .download(&attachment_row.s3_file_path)
            .await
            .map_err(|e| match e {
                StorageError::ObjectNotFound => MailError::MailAttachmentNotFound,
                _ => MailError::Internal,
            })?;

        Ok((bytes, attachment_row.file_name))
    }

    #[tracing::instrument(
        name = "mail_service.send_email_change_notification",
        skip(self),
        fields(driver_id = %driver.pk_driver_id)
    )]
    async fn send_email_change_notification(&self, driver: DriverRow) -> Result<(), MailError> {
        let bitmask = self
            .mail_database_repository
            .get_driver_mail_preferences(driver.pk_driver_id)
            .await?;

        let bit = 1 << (EnumDriverMailType::AccountChangement.as_id() - 1);
        if bitmask & bit == 0 {
            return Ok(());
        }

        let mail = self
            .mail_database_repository
            .create_mail(
                driver.clone(),
                EnumDriverMailType::AccountChangement,
                "Driver email change notification".to_string(),
                None,
            )
            .await?;

        match self
            .mail_smtp_repository
            .send_driver_email_change_email(driver.clone())
            .await
        {
            Ok(_) => {
                self.mail_database_repository
                    .update_mail_status(
                        mail.pk_driver_mail_id,
                        MailStatus::SUCCESS,
                        Some(Utc::now()),
                    )
                    .await?;
            }
            Err(_) => {
                let _ = self
                    .mail_database_repository
                    .update_mail_status(mail.pk_driver_mail_id, MailStatus::FAILED, None)
                    .await?;
                return Err(MailError::Internal);
            }
        }

        Ok(())
    }

    #[tracing::instrument(
        name = "mail_service.send_password_change_notification",
        skip(self),
        fields(driver_id = %driver.pk_driver_id)
    )]
    async fn send_password_change_notification(&self, driver: DriverRow) -> Result<(), MailError> {
        let bitmask = self
            .mail_database_repository
            .get_driver_mail_preferences(driver.pk_driver_id)
            .await?;

        let bit = 1 << (EnumDriverMailType::AccountChangement.as_id() - 1);
        if bitmask & bit == 0 {
            return Ok(());
        }

        let mail = self
            .mail_database_repository
            .create_mail(
                driver.clone(),
                EnumDriverMailType::AccountChangement,
                "Driver password change notification".to_string(),
                None,
            )
            .await?;

        match self
            .mail_smtp_repository
            .send_driver_password_change_email(driver.clone())
            .await
        {
            Ok(_) => {
                self.mail_database_repository
                    .update_mail_status(
                        mail.pk_driver_mail_id,
                        MailStatus::SUCCESS,
                        Some(Utc::now()),
                    )
                    .await?;
            }
            Err(_) => {
                let _ = self
                    .mail_database_repository
                    .update_mail_status(mail.pk_driver_mail_id, MailStatus::FAILED, None)
                    .await?;
                return Err(MailError::Internal);
            }
        }

        Ok(())
    }

    #[tracing::instrument(
        name = "mail_service.send_reset_password_email",
        skip(self),
        fields(driver_id = %driver.pk_driver_id)
    )]
    async fn send_reset_password_email(&self, driver: DriverRow) -> Result<(), MailError> {
        let reset_value = self
            .driver_cache_repository
            .generate_random_value(100)
            .await
            .map_err(|_| MailError::Internal)?;

        let (redis_key, redis_ttl) = self
            .driver_cache_repository
            .get_key_by_type(driver.pk_driver_id, DriverCacheKeyType::ResetPassword);
        let _ = self
            .driver_cache_repository
            .set_redis(redis_key, reset_value.clone(), redis_ttl)
            .await;

        let mail = self
            .mail_database_repository
            .create_mail(
                driver.clone(),
                EnumDriverMailType::PasswordReset,
                "Driver password reset".to_string(),
                None,
            )
            .await?;

        match self
            .mail_smtp_repository
            .send_driver_reset_password_email(driver.clone(), reset_value, redis_ttl)
            .await
        {
            Ok(_) => {
                self.mail_database_repository
                    .update_mail_status(
                        mail.pk_driver_mail_id,
                        MailStatus::SUCCESS,
                        Some(Utc::now()),
                    )
                    .await?;
            }
            Err(_) => {
                let _ = self
                    .mail_database_repository
                    .update_mail_status(mail.pk_driver_mail_id, MailStatus::FAILED, None)
                    .await?;
                return Err(MailError::Internal);
            }
        }

        Ok(())
    }

    #[tracing::instrument(
        name = "mail_service.send_email_change_verification",
        skip(self),
        fields(
            driver_id = %driver.pk_driver_id,
        )
    )]
    async fn send_email_change_verification(&self, driver: DriverRow) -> Result<(), MailError> {
        let verify_value = self
            .driver_cache_repository
            .generate_random_value(100)
            .await
            .map_err(|_| MailError::Internal)?;

        let (redis_key, redis_ttl) = self
            .driver_cache_repository
            .get_key_by_type(driver.pk_driver_id, DriverCacheKeyType::VerifyEmail);
        let _ = self
            .driver_cache_repository
            .set_redis(redis_key, verify_value.clone(), redis_ttl)
            .await;

        let mail = self
            .mail_database_repository
            .create_mail(
                driver.clone(),
                EnumDriverMailType::AccountChangement,
                "Driver email change verification".to_string(),
                None,
            )
            .await?;

        match self
            .mail_smtp_repository
            .send_driver_verification_email(driver.clone(), verify_value, redis_ttl)
            .await
        {
            Ok(_) => {
                self.mail_database_repository
                    .update_mail_status(
                        mail.pk_driver_mail_id,
                        MailStatus::SUCCESS,
                        Some(Utc::now()),
                    )
                    .await?;
            }
            Err(_) => {
                let _ = self
                    .mail_database_repository
                    .update_mail_status(mail.pk_driver_mail_id, MailStatus::FAILED, None)
                    .await?;

                return Err(MailError::Internal);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use chrono::{DateTime, Utc};
    use uuid::Uuid;

    use crate::{
        Service, ServiceConfig,
        domain::{
            common::constants::EnumDriverMailType,
            document::port::MockDocumentExternalRepository,
            driver::{
                entities::DriverRow,
                port::{MockDriverCacheRepository, MockDriverDatabaseRepository},
            },
            health::port::MockHealthRepository,
            mail::{
                entities::{
                    DriverMailAttachmentRow, DriverMailRow, DriverMailTypeRow, MailStatus,
                },
                port::{MailDatabaseRepository, MailService, MockMailCacheRepository, MockMailSmtpRepository},
            },
            storage::port::MockStorageRepository,
            update::port::{MockUpdateCacheRepository, MockUpdateDatabaseRepository},
            workday::port::{MockWorkdayCacheRepository, MockWorkdayDatabaseRepository},
        },
        infrastructure::mail::repositories::error::MailError,
    };

    /// Spy for MailDatabaseRepository: lets tests control the preference bitmask
    /// and observe how many times create_mail was called.
    #[derive(Clone)]
    struct MailDbSpy {
        preferences_bitmask: i32,
        create_mail_calls: Arc<Mutex<u32>>,
        mails: Arc<Mutex<Vec<DriverMailRow>>>,
    }

    impl MailDbSpy {
        fn new(preferences_bitmask: i32) -> Self {
            Self {
                preferences_bitmask,
                create_mail_calls: Arc::new(Mutex::new(0)),
                mails: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn create_mail_call_count(&self) -> u32 {
            *self.create_mail_calls.lock().unwrap()
        }
    }

    impl MailDatabaseRepository for MailDbSpy {
        async fn create_mail(
            &self,
            driver: DriverRow,
            mail_type: EnumDriverMailType,
            description: String,
            content: Option<String>,
        ) -> Result<DriverMailRow, MailError> {
            *self.create_mail_calls.lock().unwrap() += 1;
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
            self.mails.lock().unwrap().push(mail.clone());
            Ok(mail)
        }

        async fn update_mail_status(
            &self,
            mail_id: Uuid,
            status: MailStatus,
            sent_at: Option<DateTime<Utc>>,
        ) -> Result<DriverMailRow, MailError> {
            let mails = self.mails.lock().unwrap();
            mails
                .iter()
                .find(|m| m.pk_driver_mail_id == mail_id)
                .map(|m| DriverMailRow {
                    status: status.clone(),
                    sent_at,
                    ..m.clone()
                })
                .ok_or(MailError::MailNotFound)
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

        async fn get_mail_type_by_id(&self, _mail_type_id: i32) -> Result<DriverMailTypeRow, MailError> {
            Err(MailError::MailTypeNotFound)
        }

        async fn get_driver_mail_preferences(&self, _driver_id: Uuid) -> Result<i32, MailError> {
            Ok(self.preferences_bitmask)
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

        async fn has_monthly_report_this_month(
            &self,
            _driver_id: Uuid,
            _month: u32,
            _year: i32,
        ) -> Result<bool, MailError> {
            Ok(false)
        }

        async fn has_document_at_path(&self, _s3_file_path: &str) -> Result<bool, MailError> {
            Ok(false)
        }

        async fn create_document(
            &self,
            _s3_file_path: String,
            _file_name: String,
        ) -> Result<Uuid, MailError> {
            Ok(Uuid::new_v4())
        }

        async fn create_mail_attachment(
            &self,
            _mail_id: Uuid,
            _document_id: Uuid,
        ) -> Result<(), MailError> {
            Ok(())
        }
    }

    fn make_driver() -> DriverRow {
        DriverRow {
            pk_driver_id: Uuid::new_v4(),
            firstname: "Test".to_string(),
            lastname: "Driver".to_string(),
            gender: None,
            email: "test@example.be".to_string(),
            password_hash: "hash".to_string(),
            phone_number: None,
            is_searchable: true,
            allow_request_professional_agreement: false,
            language: "fr".to_string(),
            rest_json: None,
            mail_preferences: 0,
            created_at: Utc::now(),
            verified_at: None,
            last_login_at: None,
            deactivated_at: None,
        }
    }

    fn make_service(
        mail_db: MailDbSpy,
    ) -> Service<
        MockHealthRepository,
        MockDriverDatabaseRepository,
        MockDriverCacheRepository,
        MockWorkdayDatabaseRepository,
        MockWorkdayCacheRepository,
        MockMailSmtpRepository,
        MailDbSpy,
        MockMailCacheRepository,
        MockUpdateDatabaseRepository,
        MockUpdateCacheRepository,
        MockDocumentExternalRepository,
        MockStorageRepository,
    > {
        Service::new(
            MockHealthRepository,
            MockDriverDatabaseRepository::new(),
            MockDriverCacheRepository::new(),
            MockWorkdayDatabaseRepository::new(),
            MockWorkdayCacheRepository::new(),
            MockMailSmtpRepository,
            mail_db,
            MockMailCacheRepository::new(),
            MockUpdateDatabaseRepository::new(),
            MockUpdateCacheRepository::new(),
            MockDocumentExternalRepository,
            MockStorageRepository::new(),
            ServiceConfig {
                workday_garbage_retention_days: 30,
                account_deactivation_days: 30,
            },
        )
    }

    // ── send_creation_email ──────────────────────────────────────────────────────

    #[tokio::test]
    async fn send_creation_email_returns_err_when_account_verification_disabled() {
        let spy = MailDbSpy::new(0);
        let service = make_service(spy.clone());

        let result = service.send_creation_email(make_driver()).await;

        assert!(
            matches!(result, Err(MailError::MailPreferenceDisabled)),
            "expected MailPreferenceDisabled, got {:?}",
            result
        );
        assert_eq!(spy.create_mail_call_count(), 0, "no mail should be created");
    }

    #[tokio::test]
    async fn send_creation_email_succeeds_and_creates_mail_when_preference_enabled() {
        let all_enabled = i32::MAX;
        let spy = MailDbSpy::new(all_enabled);
        let service = make_service(spy.clone());

        let result = service.send_creation_email(make_driver()).await;

        assert!(result.is_ok(), "expected Ok, got {:?}", result);
        assert_eq!(spy.create_mail_call_count(), 1, "one mail should be created");
    }

    // ── send_email_change_notification ──────────────────────────────────────────

    #[tokio::test]
    async fn send_email_change_notification_skips_silently_when_preference_disabled() {
        let spy = MailDbSpy::new(0);
        let service = make_service(spy.clone());

        let result = service.send_email_change_notification(make_driver()).await;

        assert!(result.is_ok(), "expected Ok(()) silent skip, got {:?}", result);
        assert_eq!(spy.create_mail_call_count(), 0, "no mail should be created");
    }

    #[tokio::test]
    async fn send_email_change_notification_creates_mail_when_preference_enabled() {
        let account_changement_bit = 1 << (EnumDriverMailType::AccountChangement.as_id() - 1);
        let spy = MailDbSpy::new(account_changement_bit);
        let service = make_service(spy.clone());

        let result = service.send_email_change_notification(make_driver()).await;

        assert!(result.is_ok(), "expected Ok, got {:?}", result);
        assert_eq!(spy.create_mail_call_count(), 1, "one mail should be created");
    }

    // ── send_password_change_notification ───────────────────────────────────────

    #[tokio::test]
    async fn send_password_change_notification_skips_silently_when_preference_disabled() {
        let spy = MailDbSpy::new(0);
        let service = make_service(spy.clone());

        let result = service.send_password_change_notification(make_driver()).await;

        assert!(result.is_ok(), "expected Ok(()) silent skip, got {:?}", result);
        assert_eq!(spy.create_mail_call_count(), 0, "no mail should be created");
    }

    #[tokio::test]
    async fn send_password_change_notification_creates_mail_when_preference_enabled() {
        let account_changement_bit = 1 << (EnumDriverMailType::AccountChangement.as_id() - 1);
        let spy = MailDbSpy::new(account_changement_bit);
        let service = make_service(spy.clone());

        let result = service.send_password_change_notification(make_driver()).await;

        assert!(result.is_ok(), "expected Ok, got {:?}", result);
        assert_eq!(spy.create_mail_call_count(), 1, "one mail should be created");
    }
}
