use chrono::{Datelike, Months, Utc};
use tracing::{error, info, warn};

use plannify_driver_api_core::{
    application::{DriverRepositories, DriverService},
    domain::{
        common::constants::EnumDriverMailType,
        driver::port::DriverDatabaseRepository,
        mail::{
            entities::MailStatus,
            port::{MailDatabaseRepository, MailSmtpRepository},
        },
        storage::port::StorageRepository,
        workday::port::WorkdayService,
    },
};

pub async fn run(repos: &DriverRepositories) -> i32 {
    let service: DriverService = repos.clone().into();
    run_inner(
        &repos.driver_database_repository,
        &repos.mail_database_repository,
        &repos.mail_smtp_repository,
        &repos.storage_repository,
        &service,
    )
    .await
}

async fn run_inner<DDB, MDB, MS, SR, WS>(
    driver_db: &DDB,
    mail_db: &MDB,
    mail_smtp: &MS,
    storage: &SR,
    workday_service: &WS,
) -> i32
where
    DDB: DriverDatabaseRepository,
    MDB: MailDatabaseRepository,
    MS: MailSmtpRepository,
    SR: StorageRepository,
    WS: WorkdayService,
{
    info!("Starting send_monthly_reports job");

    let today = Utc::now().date_naive();
    let prev = today - Months::new(1);
    let month = prev.month() as i32;
    let year = prev.year();

    info!("Targeting reports for {}/{}", month, year);

    let drivers = match driver_db.get_drivers_with_monthly_report_preference().await {
        Ok(d) => d,
        Err(e) => {
            error!(
                "Failed to get drivers with monthly report preference: {}",
                e
            );
            return 1;
        }
    };

    let total = drivers.len();
    if total == 0 {
        info!("No drivers with monthly report preference enabled");
        return 0;
    }

    info!("Found {} driver(s) with monthly report preference", total);

    let mut sent = 0u32;
    let mut skipped = 0u32;
    let mut failed = 0u32;

    for driver in &drivers {
        let driver_id = driver.pk_driver_id;

        match mail_db
            .has_monthly_report_this_month(driver_id, month as u32, year)
            .await
        {
            Ok(true) => {
                warn!(
                    driver_id = %driver_id,
                    "Monthly report already sent for {}/{}, skipping",
                    month,
                    year
                );
                skipped += 1;
                continue;
            }
            Ok(false) => {}
            Err(e) => {
                failed += 1;
                error!(
                    driver_id = %driver_id,
                    error = ?e,
                    "Failed to check monthly report idempotence"
                );
                continue;
            }
        }

        let s3_key = format!(
            "drivers/{}/mails/monthly-{}-{:02}.pdf",
            driver_id, year, month
        );

        match mail_db.has_document_at_path(&s3_key).await {
            Ok(true) => {
                warn!(
                    driver_id = %driver_id,
                    "Document already exists at {} for mail type 4, skipping",
                    s3_key
                );
                skipped += 1;
                continue;
            }
            Ok(false) => {}
            Err(e) => {
                failed += 1;
                error!(
                    driver_id = %driver_id,
                    error = ?e,
                    "Failed to check existing document"
                );
                continue;
            }
        }

        let pdf = match workday_service
            .get_workday_document_by_month(driver_id, month, year)
            .await
        {
            Ok(Some(bytes)) => bytes,
            Ok(None) => {
                info!(
                    driver_id = %driver_id,
                    "No workdays found for {}/{}, skipping",
                    month,
                    year
                );
                skipped += 1;
                continue;
            }
            Err(e) => {
                failed += 1;
                error!(
                    driver_id = %driver_id,
                    error = ?e,
                    "Failed to get workday document for {}/{}",
                    month,
                    year
                );
                continue;
            }
        };

        let file_name = format!("workdays-{}-{:02}.pdf", year, month);

        if let Err(e) = storage
            .upload(&s3_key, pdf.clone(), "application/pdf")
            .await
        {
            failed += 1;
            error!(
                driver_id = %driver_id,
                error = ?e,
                "Failed to upload monthly report PDF to S3"
            );
            continue;
        }

        let document_id = match mail_db.create_document(s3_key, file_name.clone()).await {
            Ok(id) => id,
            Err(e) => {
                failed += 1;
                error!(
                    driver_id = %driver_id,
                    error = ?e,
                    "Failed to create document record"
                );
                continue;
            }
        };

        let description = format!("Rapport mensuel {}/{}", month, year);
        let mail = match mail_db
            .create_mail(
                driver.clone(),
                EnumDriverMailType::MonthlyReports,
                description,
                None,
            )
            .await
        {
            Ok(m) => m,
            Err(e) => {
                failed += 1;
                error!(
                    driver_id = %driver_id,
                    error = ?e,
                    "Failed to create mail record"
                );
                continue;
            }
        };

        if let Err(e) = mail_db
            .create_mail_attachment(mail.pk_driver_mail_id, document_id)
            .await
        {
            failed += 1;
            error!(
                driver_id = %driver_id,
                error = ?e,
                "Failed to create mail attachment record"
            );
            continue;
        }

        let send_result = mail_smtp
            .send_driver_monthly_report_email(driver.clone(), month as u32, year, pdf, file_name)
            .await;

        match send_result {
            Ok(()) => {
                let _ = mail_db
                    .update_mail_status(
                        mail.pk_driver_mail_id,
                        MailStatus::SUCCESS,
                        Some(Utc::now()),
                    )
                    .await;
                sent += 1;
                info!(driver_id = %driver_id, "Monthly report sent for {}/{}", month, year);
            }
            Err(e) => {
                let _ = mail_db
                    .update_mail_status(mail.pk_driver_mail_id, MailStatus::FAILED, None)
                    .await;
                failed += 1;
                error!(
                    driver_id = %driver_id,
                    error = ?e,
                    "Failed to send monthly report email"
                );
            }
        }
    }

    info!(
        total,
        sent, skipped, failed, "send_monthly_reports job completed"
    );

    if failed > 0 { 1 } else { 0 }
}

#[cfg(test)]
mod tests {
    use super::run_inner;
    use bytes::Bytes;
    use chrono::NaiveDate;
    use plannify_driver_api_core::{
        domain::{
            driver::{
                entities::{CreateDriverRequest, DriverLanguage, DriverRow},
                port::{DriverDatabaseRepository, MockDriverDatabaseRepository},
            },
            mail::port::{MailSmtpRepository, MockMailDatabaseRepository},
            storage::port::StorageRepository,
            workday::{
                entities::{
                    CreateWorkdayRequest, UpdateWorkdayRequest, Workday,
                    WorkdayDocumentInformation, WorkdayGarbageRow, WorkdayRow,
                },
                port::WorkdayService,
            },
        },
        infrastructure::{
            mail::repositories::error::MailError, storage::repositories::error::StorageError,
            workday::repositories::error::WorkdayError,
        },
    };
    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
        time::Duration,
    };
    use uuid::Uuid;

    // ── Stubs ──────────────────────────────────────────────────────────────────

    struct StubStorage {
        fail: bool,
        store: Arc<Mutex<HashMap<String, Bytes>>>,
    }

    impl StubStorage {
        fn ok() -> Self {
            Self {
                fail: false,
                store: Default::default(),
            }
        }
        fn failing() -> Self {
            Self {
                fail: true,
                store: Default::default(),
            }
        }
    }

    impl StorageRepository for StubStorage {
        async fn upload(&self, key: &str, data: Bytes, _: &str) -> Result<(), StorageError> {
            if self.fail {
                return Err(StorageError::UploadError("test".into()));
            }
            self.store.lock().unwrap().insert(key.to_string(), data);
            Ok(())
        }
        async fn download(&self, _: &str) -> Result<Bytes, StorageError> {
            unreachable!()
        }
        async fn delete(&self, _: &str) -> Result<(), StorageError> {
            unreachable!()
        }
        async fn generate_presigned_url(
            &self,
            _: &str,
            _: Duration,
        ) -> Result<String, StorageError> {
            unreachable!()
        }
    }

    struct StubWorkdayService {
        pdf: Option<Bytes>,
        fail: bool,
    }

    impl StubWorkdayService {
        fn returns_pdf(bytes: Bytes) -> Self {
            Self {
                pdf: Some(bytes),
                fail: false,
            }
        }
        fn returns_none() -> Self {
            Self {
                pdf: None,
                fail: false,
            }
        }
        fn failing() -> Self {
            Self {
                pdf: None,
                fail: true,
            }
        }
    }

    impl WorkdayService for StubWorkdayService {
        async fn get_workday_document_by_month(
            &self,
            _: Uuid,
            _: i32,
            _: i32,
        ) -> Result<Option<Bytes>, WorkdayError> {
            if self.fail {
                return Err(WorkdayError::Internal);
            }
            Ok(self.pdf.clone())
        }
        async fn get_workday_by_date(
            &self,
            _: Uuid,
            _: NaiveDate,
        ) -> Result<Workday, WorkdayError> {
            unreachable!()
        }
        async fn get_workdays_by_month(
            &self,
            _: Uuid,
            _: i32,
            _: i32,
        ) -> Result<Vec<Workday>, WorkdayError> {
            unreachable!()
        }
        async fn get_workdays_by_period(
            &self,
            _: Uuid,
            _: NaiveDate,
            _: NaiveDate,
            _: u32,
            _: u32,
        ) -> Result<(Vec<Workday>, u32), WorkdayError> {
            unreachable!()
        }
        async fn create_workday(
            &self,
            _: Uuid,
            _: CreateWorkdayRequest,
        ) -> Result<WorkdayRow, WorkdayError> {
            unreachable!()
        }
        async fn update_workday(
            &self,
            _: Uuid,
            _: UpdateWorkdayRequest,
        ) -> Result<WorkdayRow, WorkdayError> {
            unreachable!()
        }
        async fn delete_workday(&self, _: Uuid, _: NaiveDate) -> Result<(), WorkdayError> {
            unreachable!()
        }
        async fn get_workdays_garbage(
            &self,
            _: Uuid,
        ) -> Result<Vec<WorkdayGarbageRow>, WorkdayError> {
            unreachable!()
        }
        async fn create_workday_garbage(
            &self,
            _: Uuid,
            _: NaiveDate,
        ) -> Result<WorkdayGarbageRow, WorkdayError> {
            unreachable!()
        }
        async fn delete_workday_garbage(&self, _: Uuid, _: NaiveDate) -> Result<(), WorkdayError> {
            unreachable!()
        }
        async fn get_workday_documents(&self, _: Uuid) -> Result<Vec<i32>, WorkdayError> {
            unreachable!()
        }
        async fn get_generated_document_by_year(
            &self,
            _: Uuid,
            _: i32,
        ) -> Result<Vec<WorkdayDocumentInformation>, WorkdayError> {
            unreachable!()
        }
        async fn get_workday_documents_by_year(
            &self,
            _: Uuid,
            _: i32,
        ) -> Result<Vec<WorkdayDocumentInformation>, WorkdayError> {
            unreachable!()
        }
    }

    struct MockMailSmtpRepository;

    impl MailSmtpRepository for MockMailSmtpRepository {
        fn send_email(&self, _: String, _: String, _: String) -> Result<(), MailError> {
            Ok(())
        }
        async fn send_driver_creation_email(
            &self,
            _: DriverRow,
            _: String,
            _: u64,
        ) -> Result<(), MailError> {
            Ok(())
        }
        async fn send_driver_verification_email(
            &self,
            _: DriverRow,
            _: String,
            _: u64,
        ) -> Result<(), MailError> {
            Ok(())
        }
        async fn send_driver_email_change_email(&self, _: DriverRow) -> Result<(), MailError> {
            Ok(())
        }
        async fn send_driver_password_change_email(&self, _: DriverRow) -> Result<(), MailError> {
            Ok(())
        }
        async fn send_driver_deactivation_email(&self, _: DriverRow) -> Result<(), MailError> {
            Ok(())
        }
        async fn send_driver_reactivation_email(&self, _: DriverRow) -> Result<(), MailError> {
            Ok(())
        }
        async fn send_driver_reset_password_email(
            &self,
            _: DriverRow,
            _: String,
            _: u64,
        ) -> Result<(), MailError> {
            Ok(())
        }
        async fn send_driver_monthly_report_email(
            &self,
            _: DriverRow,
            _: u32,
            _: i32,
            _: Bytes,
            _: String,
        ) -> Result<(), MailError> {
            Ok(())
        }
    }

    // ── Helpers ────────────────────────────────────────────────────────────────

    async fn driver_with_monthly_pref(db: &MockDriverDatabaseRepository, email: &str) -> DriverRow {
        let driver = db
            .create_driver(CreateDriverRequest {
                firstname: "Test".into(),
                lastname: "Driver".into(),
                gender: None,
                email: email.into(),
                password: "hashed".into(),
                language: DriverLanguage::FR,
            })
            .await
            .unwrap();
        db.update_driver(DriverRow {
            mail_preferences: 8,
            ..driver
        })
        .await
        .unwrap()
    }

    // ── Tests ──────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_no_drivers_returns_success() {
        let result = run_inner(
            &MockDriverDatabaseRepository::new(),
            &MockMailDatabaseRepository::new(),
            &MockMailSmtpRepository,
            &StubStorage::ok(),
            &StubWorkdayService::returns_none(),
        )
        .await;
        assert_eq!(result, 0);
    }

    #[tokio::test]
    async fn test_driver_with_no_workdays_skipped() {
        let driver_db = MockDriverDatabaseRepository::new();
        driver_with_monthly_pref(&driver_db, "alice@example.com").await;

        let result = run_inner(
            &driver_db,
            &MockMailDatabaseRepository::new(),
            &MockMailSmtpRepository,
            &StubStorage::ok(),
            &StubWorkdayService::returns_none(),
        )
        .await;
        assert_eq!(result, 0);
    }

    #[tokio::test]
    async fn test_happy_path_sends_report() {
        let driver_db = MockDriverDatabaseRepository::new();
        driver_with_monthly_pref(&driver_db, "bob@example.com").await;

        let result = run_inner(
            &driver_db,
            &MockMailDatabaseRepository::new(),
            &MockMailSmtpRepository,
            &StubStorage::ok(),
            &StubWorkdayService::returns_pdf(Bytes::from("fake-pdf")),
        )
        .await;
        assert_eq!(result, 0);
    }

    #[tokio::test]
    async fn test_s3_failure_returns_error() {
        let driver_db = MockDriverDatabaseRepository::new();
        driver_with_monthly_pref(&driver_db, "carol@example.com").await;

        let result = run_inner(
            &driver_db,
            &MockMailDatabaseRepository::new(),
            &MockMailSmtpRepository,
            &StubStorage::failing(),
            &StubWorkdayService::returns_pdf(Bytes::from("pdf")),
        )
        .await;
        assert_eq!(result, 1);
    }

    #[tokio::test]
    async fn test_workday_service_failure_returns_error() {
        let driver_db = MockDriverDatabaseRepository::new();
        driver_with_monthly_pref(&driver_db, "dave@example.com").await;

        let result = run_inner(
            &driver_db,
            &MockMailDatabaseRepository::new(),
            &MockMailSmtpRepository,
            &StubStorage::ok(),
            &StubWorkdayService::failing(),
        )
        .await;
        assert_eq!(result, 1);
    }
}
