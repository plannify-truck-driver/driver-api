use plannify_driver_api_core::{
    application::DriverRepositories, domain::workday::port::WorkdayDatabaseRepository,
};
use tracing::{error, info};

pub async fn run(repos: &DriverRepositories) -> i32 {
    run_inner(&repos.workday_database_repository).await
}

async fn run_inner<WDB: WorkdayDatabaseRepository>(workday_db: &WDB) -> i32 {
    info!("Starting delete_garbage job");

    let count = match workday_db.delete_definitly_workday_garbage().await {
        Ok(count) => count,
        Err(e) => {
            error!(error = ?e, "Failed to delete expired garbage entries");
            return 1;
        }
    };

    if count == 0 {
        info!("No expired garbage entries found");
    } else {
        info!("{} workdays deleted", count);
    }

    0
}

#[cfg(test)]
mod tests {
    use super::run_inner;
    use chrono::{DateTime, NaiveDate, Utc};
    use plannify_driver_api_core::{
        domain::workday::{
            entities::{
                CreateWorkdayRequest, UpdateWorkdayRequest, WorkdayDocument,
                WorkdayDocumentInformation, WorkdayGarbageRow, WorkdayRow,
            },
            port::WorkdayDatabaseRepository,
        },
        infrastructure::workday::repositories::error::WorkdayError,
    };
    use uuid::Uuid;

    struct StubWorkdayDb {
        delete_result: Result<u32, WorkdayError>,
    }

    impl StubWorkdayDb {
        fn returns_count(n: u32) -> Self {
            Self {
                delete_result: Ok(n),
            }
        }
        fn fails() -> Self {
            Self {
                delete_result: Err(WorkdayError::DatabaseError),
            }
        }
    }

    impl WorkdayDatabaseRepository for StubWorkdayDb {
        async fn delete_definitly_workday_garbage(&self) -> Result<u32, WorkdayError> {
            self.delete_result.clone()
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
        async fn create_workday_document(
            &self,
            _: Uuid,
            _: i32,
            _: i32,
            _: String,
            _: String,
        ) -> Result<WorkdayDocument, WorkdayError> {
            unreachable!()
        }
        async fn get_pending_document_months(
            &self,
            _: NaiveDate,
        ) -> Result<Vec<(Uuid, i32, i32)>, WorkdayError> {
            unreachable!()
        }
    }

    #[tokio::test]
    async fn test_no_expired_entries_returns_success() {
        assert_eq!(run_inner(&StubWorkdayDb::returns_count(0)).await, 0);
    }

    #[tokio::test]
    async fn test_deleted_entries_returns_success() {
        assert_eq!(run_inner(&StubWorkdayDb::returns_count(3)).await, 0);
    }

    #[tokio::test]
    async fn test_db_error_returns_failure() {
        assert_eq!(run_inner(&StubWorkdayDb::fails()).await, 1);
    }
}
