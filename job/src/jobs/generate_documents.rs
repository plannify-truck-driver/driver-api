use chrono::{Datelike, Months, NaiveDate, Utc};
use plannify_driver_api_core::{
    application::{DriverRepositories, DriverService},
    domain::{
        storage::port::StorageRepository,
        workday::port::{WorkdayCacheRepository, WorkdayDatabaseRepository, WorkdayService},
    },
};
use tracing::{error, info, warn};

pub async fn run(repos: &DriverRepositories, months_ago: u32) -> i32 {
    let service: DriverService = repos.clone().into();
    run_inner(
        &repos.workday_database_repository,
        &repos.workday_cache_repository,
        &repos.storage_repository,
        &service,
        months_ago,
    )
    .await
}

async fn run_inner<WDB, WC, SR, WS>(
    workday_db: &WDB,
    cache: &WC,
    storage: &SR,
    service: &WS,
    months_ago: u32,
) -> i32
where
    WDB: WorkdayDatabaseRepository,
    WC: WorkdayCacheRepository,
    SR: StorageRepository,
    WS: WorkdayService,
{
    info!("Starting generate_documents job");

    let today = Utc::now().date_naive();
    let cutoff = today - Months::new(months_ago);
    let before = NaiveDate::from_ymd_opt(cutoff.year(), cutoff.month(), 1)
        .expect("cutoff date is always valid");

    info!("Looking for months with workdays before {}", before);

    let pending = match workday_db.get_pending_document_months(before).await {
        Ok(p) => p,
        Err(e) => {
            error!("Failed to fetch pending document months: {}", e);
            return 1;
        }
    };

    let total = pending.len();
    if total == 0 {
        info!("No pending documents to generate");
        return 0;
    }

    info!("Found {} month(s) pending document generation", total);

    let mut generated = 0u32;
    let mut failed = 0u32;

    for (driver_id, month, year) in &pending {
        let pdf = match service
            .get_workday_document_by_month(*driver_id, *month, *year)
            .await
        {
            Ok(Some(bytes)) => bytes,
            Ok(None) => {
                warn!(
                    driver_id = %driver_id,
                    month = %month,
                    year = %year,
                    "No workdays found for this month"
                );
                continue;
            }
            Err(e) => {
                failed += 1;
                error!(
                    driver_id = %driver_id,
                    month = %month,
                    year = %year,
                    error = ?e,
                    "Failed to generate document"
                );
                continue;
            }
        };

        let s3_key = format!(
            "drivers/{}/workdays/monthly-reports/workdays-{}-{:02}.pdf",
            driver_id, year, month
        );
        let file_name = format!("workdays-{}-{:02}.pdf", year, month);

        if let Err(e) = storage.upload(&s3_key, pdf, "application/pdf").await {
            failed += 1;
            error!(
                driver_id = %driver_id,
                month = %month,
                year = %year,
                error = ?e,
                "Failed to upload document to S3"
            );
            continue;
        }

        let doc = match workday_db
            .create_workday_document(*driver_id, *month, *year, s3_key, file_name)
            .await
        {
            Ok(doc) => doc,
            Err(e) => {
                failed += 1;
                error!(
                    driver_id = %driver_id,
                    month = %month,
                    year = %year,
                    error = ?e,
                    "Failed to persist document record"
                );
                continue;
            }
        };

        let _ = cache
            .set_workday_document_record(*driver_id, *month, *year, Some(doc))
            .await;
        let _ = cache.delete_document_years(*driver_id).await;
        let _ = cache
            .delete_generated_documents_by_year(*driver_id, *year)
            .await;
        let _ = cache.delete_documents_by_year(*driver_id, *year).await;

        generated += 1;
        info!(
            driver_id = %driver_id,
            month = %month,
            year = %year,
            "Document generated and stored"
        );
    }

    info!(total, generated, failed, "generate_documents job completed");

    if failed > 0 { 1 } else { 0 }
}

#[cfg(test)]
mod tests {
    use super::run_inner;
    use bytes::Bytes;
    use chrono::{DateTime, NaiveDate, Utc};
    use plannify_driver_api_core::{
        domain::{
            storage::port::StorageRepository,
            workday::{
                entities::{
                    CreateWorkdayRequest, UpdateWorkdayRequest, Workday, WorkdayDocument,
                    WorkdayDocumentInformation, WorkdayGarbageRow, WorkdayRow,
                },
                port::{MockWorkdayCacheRepository, WorkdayDatabaseRepository, WorkdayService},
            },
        },
        infrastructure::{
            storage::repositories::error::StorageError, workday::repositories::error::WorkdayError,
        },
    };
    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
        time::Duration,
    };
    use uuid::Uuid;

    // --- Stubs ---

    type UploadLog = Arc<Mutex<Vec<(Uuid, i32, i32, String)>>>;

    struct StubWorkdayDb {
        pending: Vec<(Uuid, i32, i32)>,
        create_doc_fail: bool,
        uploads: UploadLog,
    }

    impl StubWorkdayDb {
        fn empty() -> Self {
            Self {
                pending: vec![],
                create_doc_fail: false,
                uploads: Default::default(),
            }
        }
        fn with_pending(driver_id: Uuid, month: i32, year: i32) -> Self {
            Self {
                pending: vec![(driver_id, month, year)],
                create_doc_fail: false,
                uploads: Default::default(),
            }
        }
        fn with_pending_and_failing_persist(driver_id: Uuid, month: i32, year: i32) -> Self {
            Self {
                pending: vec![(driver_id, month, year)],
                create_doc_fail: true,
                uploads: Default::default(),
            }
        }
    }

    impl WorkdayDatabaseRepository for StubWorkdayDb {
        async fn get_pending_document_months(
            &self,
            _: NaiveDate,
        ) -> Result<Vec<(Uuid, i32, i32)>, WorkdayError> {
            Ok(self.pending.clone())
        }
        async fn create_workday_document(
            &self,
            driver_id: Uuid,
            month: i32,
            year: i32,
            s3_key: String,
            _: String,
        ) -> Result<WorkdayDocument, WorkdayError> {
            if self.create_doc_fail {
                return Err(WorkdayError::DatabaseError);
            }
            self.uploads
                .lock()
                .unwrap()
                .push((driver_id, month, year, s3_key));
            Ok(WorkdayDocument {
                fk_driver_id: driver_id,
                month,
                year,
                s3_file_path: String::new(),
                file_name: String::new(),
                created_at: Utc::now(),
            })
        }
        async fn get_workday_by_date(
            &self,
            _: Uuid,
            _: NaiveDate,
        ) -> Result<Option<WorkdayRow>, WorkdayError> {
            unreachable!()
        }
        async fn get_workdays_by_month(
            &self,
            _: Uuid,
            _: i32,
            _: i32,
        ) -> Result<Vec<WorkdayRow>, WorkdayError> {
            unreachable!()
        }
        async fn get_workdays_by_period(
            &self,
            _: Uuid,
            _: NaiveDate,
            _: NaiveDate,
            _: u32,
            _: u32,
        ) -> Result<(Vec<WorkdayRow>, u32), WorkdayError> {
            unreachable!()
        }
        async fn get_workday_years(&self, _: Uuid) -> Result<Vec<i32>, WorkdayError> {
            unreachable!()
        }
        async fn get_workday_months_by_year(
            &self,
            _: Uuid,
            _: i32,
        ) -> Result<Vec<i32>, WorkdayError> {
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
            _: NaiveDate,
            _: Option<DateTime<Utc>>,
        ) -> Result<WorkdayGarbageRow, WorkdayError> {
            unreachable!()
        }
        async fn delete_workday_garbage(&self, _: Uuid, _: NaiveDate) -> Result<(), WorkdayError> {
            unreachable!()
        }
        async fn delete_definitly_workday_garbage(&self) -> Result<u32, WorkdayError> {
            unreachable!()
        }
        async fn get_workday_document_years(&self, _: Uuid) -> Result<Vec<i32>, WorkdayError> {
            unreachable!()
        }
        async fn get_workday_documents_by_year(
            &self,
            _: Uuid,
            _: i32,
        ) -> Result<Vec<WorkdayDocumentInformation>, WorkdayError> {
            unreachable!()
        }
        async fn get_workday_document_record(
            &self,
            _: Uuid,
            _: i32,
            _: i32,
        ) -> Result<Option<WorkdayDocument>, WorkdayError> {
            unreachable!()
        }
        async fn get_all_document_s3_paths(&self) -> Result<Vec<String>, WorkdayError> {
            unreachable!()
        }
        async fn get_document_s3_paths_batch(
            &self,
            _: Option<&str>,
            _: i64,
        ) -> Result<Vec<String>, WorkdayError> {
            unreachable!()
        }
        async fn delete_document_by_s3_path(&self, _: &str) -> Result<(), WorkdayError> {
            unreachable!()
        }
    }

    struct StubStorage {
        fail_upload: bool,
        store: Arc<Mutex<HashMap<String, Bytes>>>,
    }

    impl StubStorage {
        fn ok() -> Self {
            Self {
                fail_upload: false,
                store: Default::default(),
            }
        }
        fn failing() -> Self {
            Self {
                fail_upload: true,
                store: Default::default(),
            }
        }
    }

    impl StorageRepository for StubStorage {
        async fn upload(&self, key: &str, data: Bytes, _: &str) -> Result<(), StorageError> {
            if self.fail_upload {
                return Err(StorageError::UploadError("test".into()));
            }
            self.store.lock().unwrap().insert(key.to_string(), data);
            Ok(())
        }
        async fn download(&self, key: &str) -> Result<Bytes, StorageError> {
            self.store
                .lock()
                .unwrap()
                .get(key)
                .cloned()
                .ok_or(StorageError::ObjectNotFound)
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
        async fn list_objects_page(
            &self,
            _: Option<&str>,
            _: Option<String>,
        ) -> Result<(Vec<String>, Option<String>), StorageError> {
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
    }

    impl WorkdayService for StubWorkdayService {
        async fn get_workday_document_by_month(
            &self,
            _: Uuid,
            _: i32,
            _: i32,
        ) -> Result<Option<Bytes>, WorkdayError> {
            if self.fail {
                Err(WorkdayError::Internal)
            } else {
                Ok(self.pdf.clone())
            }
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

    // --- Tests ---

    #[tokio::test]
    async fn test_no_pending_months_returns_success() {
        let result = run_inner(
            &StubWorkdayDb::empty(),
            &MockWorkdayCacheRepository::new(),
            &StubStorage::ok(),
            &StubWorkdayService::returns_none(),
            3,
        )
        .await;
        assert_eq!(result, 0);
    }

    #[tokio::test]
    async fn test_service_returns_none_skips_without_failure() {
        let driver_id = Uuid::new_v4();
        let result = run_inner(
            &StubWorkdayDb::with_pending(driver_id, 1, 2020),
            &MockWorkdayCacheRepository::new(),
            &StubStorage::ok(),
            &StubWorkdayService::returns_none(),
            3,
        )
        .await;
        assert_eq!(result, 0);
    }

    #[tokio::test]
    async fn test_happy_path_generates_and_stores_document() {
        let driver_id = Uuid::new_v4();
        let pdf_bytes = Bytes::from("fake-pdf-content");
        let storage = StubStorage::ok();
        let db = StubWorkdayDb::with_pending(driver_id, 3, 2020);

        let result = run_inner(
            &db,
            &MockWorkdayCacheRepository::new(),
            &storage,
            &StubWorkdayService::returns_pdf(pdf_bytes.clone()),
            3,
        )
        .await;

        assert_eq!(result, 0);

        let expected_key = format!(
            "drivers/{}/workdays/monthly-reports/workdays-2020-03.pdf",
            driver_id
        );
        assert_eq!(storage.download(&expected_key).await.unwrap(), pdf_bytes);
        assert_eq!(db.uploads.lock().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_s3_upload_failure_returns_failure() {
        let driver_id = Uuid::new_v4();
        let result = run_inner(
            &StubWorkdayDb::with_pending(driver_id, 1, 2020),
            &MockWorkdayCacheRepository::new(),
            &StubStorage::failing(),
            &StubWorkdayService::returns_pdf(Bytes::from("pdf")),
            3,
        )
        .await;
        assert_eq!(result, 1);
    }

    #[tokio::test]
    async fn test_db_persist_failure_returns_failure() {
        let driver_id = Uuid::new_v4();
        let result = run_inner(
            &StubWorkdayDb::with_pending_and_failing_persist(driver_id, 1, 2020),
            &MockWorkdayCacheRepository::new(),
            &StubStorage::ok(),
            &StubWorkdayService::returns_pdf(Bytes::from("pdf")),
            3,
        )
        .await;
        assert_eq!(result, 1);
    }
}
