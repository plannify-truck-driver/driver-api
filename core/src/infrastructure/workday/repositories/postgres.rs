use chrono::{DateTime, NaiveDate, Utc};
use sqlx::PgPool;
use tracing::error;
use uuid::Uuid;

use crate::{
    domain::workday::{
        entities::{
            CreateWorkdayRequest, DocumentRow, UpdateWorkdayRequest, WorkdayDocument,
            WorkdayDocumentInformation, WorkdayDocumentRow, WorkdayGarbageRow, WorkdayRow,
        },
        port::WorkdayDatabaseRepository,
    },
    infrastructure::workday::repositories::error::WorkdayError,
};

#[derive(Clone)]
pub struct PostgresWorkdayRepository {
    pool: PgPool,
}

impl PostgresWorkdayRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl WorkdayDatabaseRepository for PostgresWorkdayRepository {
    #[tracing::instrument(
        name = "db.workdays.get_workday_by_date",
        skip(self),
        fields(
            driver_id = %driver_id,
            date = %date,
        )
    )]
    async fn get_workday_by_date(
        &self,
        driver_id: Uuid,
        date: NaiveDate,
    ) -> Result<Option<WorkdayRow>, WorkdayError> {
        sqlx::query_as!(
            WorkdayRow,
            r#"
            SELECT *
            FROM workdays
            WHERE date = $1
            AND fk_driver_id = $2
            AND date NOT IN (
                SELECT workday_date FROM workday_garbage
                WHERE fk_driver_id = $2
            )
            "#,
            date,
            driver_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to get workday by date: {:?}", e);
            WorkdayError::DatabaseError
        })
    }

    #[tracing::instrument(
        name = "db.workdays.get_workdays_by_month",
        skip(self),
        fields(
            driver_id = %driver_id,
            month = %month,
            year = %year,
        )
    )]
    async fn get_workdays_by_month(
        &self,
        driver_id: Uuid,
        month: i32,
        year: i32,
    ) -> Result<Vec<WorkdayRow>, WorkdayError> {
        sqlx::query_as!(
            WorkdayRow,
            r#"
            SELECT *
            FROM workdays
            WHERE EXTRACT(MONTH FROM date)::INTEGER = $1
            AND EXTRACT(YEAR FROM date)::INTEGER = $2
            AND fk_driver_id = $3
            AND date NOT IN (
                SELECT workday_date FROM workday_garbage
                WHERE fk_driver_id = $3
            )
            ORDER BY date ASC
            "#,
            month,
            year,
            driver_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to get workdays by month: {:?}", e);
            WorkdayError::DatabaseError
        })
    }

    #[tracing::instrument(
        name = "db.workdays.get_workdays_by_period",
        skip(self),
        fields(
            driver_id = %driver_id,
            start_date = %start_date,
            end_date = %end_date,
        )
    )]
    async fn get_workdays_by_period(
        &self,
        driver_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
        page: u32,
        limit: u32,
    ) -> Result<(Vec<WorkdayRow>, u32), WorkdayError> {
        let total_count_record = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM workdays
            WHERE date BETWEEN $1 AND $2
            AND fk_driver_id = $3
            AND date NOT IN (
                SELECT workday_date FROM workday_garbage
                WHERE fk_driver_id = $3
            )
            "#,
            start_date,
            end_date,
            driver_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to count workdays by period: {:?}", e);
            WorkdayError::DatabaseError
        })?;

        let workdays = sqlx::query_as!(
            WorkdayRow,
            r#"
            SELECT *
            FROM workdays
            WHERE date BETWEEN $1 AND $2
            AND fk_driver_id = $3
            AND date NOT IN (
                SELECT workday_date FROM workday_garbage
                WHERE fk_driver_id = $3
            )
            ORDER BY date ASC
            LIMIT $4 OFFSET $5
            "#,
            start_date,
            end_date,
            driver_id,
            limit as i64,
            ((page - 1) * limit) as i64,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to get workdays by period: {:?}", e);
            WorkdayError::DatabaseError
        })?;

        Ok((workdays, total_count_record.count.unwrap_or(0) as u32))
    }

    #[tracing::instrument(
        name = "db.workdays.get_workday_years",
        skip(self),
        fields(
            driver_id = %driver_id,
        )
    )]
    async fn get_workday_years(&self, driver_id: Uuid) -> Result<Vec<i32>, WorkdayError> {
        let records = sqlx::query_as::<_, (Option<i32>,)>(
            r#"
            SELECT EXTRACT(YEAR FROM date)::INTEGER as year
            FROM workdays
            WHERE fk_driver_id = $1
            AND date NOT IN (
                SELECT workday_date FROM workday_garbage
                WHERE fk_driver_id = $1
            )
            GROUP BY year
            "#,
        )
        .bind(driver_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to get workday years: {:?}", e);
            WorkdayError::DatabaseError
        })?;

        Ok(records.into_iter().filter_map(|r| r.0).collect())
    }

    #[tracing::instrument(
        name = "db.workdays.get_workday_months_by_year",
        skip(self),
        fields(
            driver_id = %driver_id,
            year = %year,
        )
    )]
    async fn get_workday_months_by_year(
        &self,
        driver_id: Uuid,
        year: i32,
    ) -> Result<Vec<i32>, WorkdayError> {
        let records = sqlx::query!(
            r#"
            SELECT EXTRACT(MONTH FROM date)::INTEGER as month
            FROM workdays
            WHERE fk_driver_id = $1
            AND EXTRACT(YEAR FROM date)::INTEGER = $2
            AND date NOT IN (
                SELECT workday_date FROM workday_garbage
                WHERE fk_driver_id = $1
            )
            GROUP BY month
            "#,
            driver_id,
            year
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to get workday months by year: {:?}", e);
            WorkdayError::DatabaseError
        })?;

        Ok(records.into_iter().filter_map(|r| r.month).collect())
    }

    #[tracing::instrument(
        name = "db.workdays.create_workday",
        skip(self),
        fields(
            driver_id = %driver_id,
            date = %create_workday_request.date,
        )
    )]
    async fn create_workday(
        &self,
        driver_id: Uuid,
        create_workday_request: CreateWorkdayRequest,
    ) -> Result<WorkdayRow, WorkdayError> {
        sqlx::query_as!(
            WorkdayRow,
            r#"
            INSERT INTO workdays (date, fk_driver_id, start_time, end_time, rest_time, overnight_rest)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
            create_workday_request.date,
            driver_id,
            create_workday_request.start_time,
            create_workday_request.end_time,
            create_workday_request.rest_time,
            create_workday_request.overnight_rest,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if e.as_database_error()
                .and_then(|db_err| db_err.code().map(|code| code == "23505"))
                .unwrap_or(false)
            {
                return WorkdayError::WorkdayAlreadyExists;
            }

            error!("Failed to create workday: {:?}", e);
            WorkdayError::DatabaseError
        })
    }

    #[tracing::instrument(
        name = "db.workdays.update_workday",
        skip(self),
        fields(
            driver_id = %driver_id,
            date = %update_workday_request.date,
        )
    )]
    async fn update_workday(
        &self,
        driver_id: Uuid,
        update_workday_request: UpdateWorkdayRequest,
    ) -> Result<WorkdayRow, WorkdayError> {
        sqlx::query_as!(
            WorkdayRow,
            r#"
            UPDATE workdays
            SET start_time = $1, end_time = $2, rest_time = $3, overnight_rest = $4
            WHERE date = $5
            AND fk_driver_id = $6
            RETURNING *
            "#,
            update_workday_request.start_time,
            update_workday_request.end_time,
            update_workday_request.rest_time,
            update_workday_request.overnight_rest,
            update_workday_request.date,
            driver_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if matches!(e, sqlx::Error::RowNotFound) {
                return WorkdayError::WorkdayNotFound;
            }

            error!("Failed to update workday: {:?}", e);
            WorkdayError::DatabaseError
        })
    }

    #[tracing::instrument(
        name = "db.workdays.delete_workday",
        skip(self),
        fields(
            driver_id = %driver_id,
            date = %date,
        )
    )]
    async fn delete_workday(&self, driver_id: Uuid, date: NaiveDate) -> Result<(), WorkdayError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM workdays
            WHERE date = $1
            AND fk_driver_id = $2
            "#,
            date,
            driver_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            if matches!(e, sqlx::Error::RowNotFound) {
                return WorkdayError::WorkdayNotFound;
            }

            error!("Failed to delete workday: {:?}", e);
            WorkdayError::DatabaseError
        })?;

        if result.rows_affected() == 0 {
            return Err(WorkdayError::WorkdayNotFound);
        }

        Ok(())
    }

    #[tracing::instrument(
        name = "db.workdays.get_workdays_garbage",
        skip(self),
        fields(
            driver_id = %driver_id,
        )
    )]
    async fn get_workdays_garbage(
        &self,
        driver_id: Uuid,
    ) -> Result<Vec<WorkdayGarbageRow>, WorkdayError> {
        sqlx::query_as!(
            WorkdayGarbageRow,
            r#"
            SELECT *
            FROM workday_garbage
            WHERE fk_driver_id = $1
            ORDER BY workday_date ASC
            "#,
            driver_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to get workdays garbage: {:?}", e);
            WorkdayError::DatabaseError
        })
    }

    #[tracing::instrument(
        name = "db.workdays.create_workday_garbage",
        skip(self),
        fields(
            driver_id = %driver_id,
            date = %date,
        )
    )]
    async fn create_workday_garbage(
        &self,
        driver_id: Uuid,
        date: NaiveDate,
        scheduled_deletion_date: NaiveDate,
        created_at: Option<DateTime<Utc>>,
    ) -> Result<WorkdayGarbageRow, WorkdayError> {
        sqlx::query_as!(
            WorkdayGarbageRow,
            r#"
            INSERT INTO workday_garbage (workday_date, fk_driver_id, created_at, scheduled_deletion_date)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
            date,
            driver_id,
            created_at.unwrap_or_else(Utc::now),
            scheduled_deletion_date,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            let db_err = e.as_database_error();

            if db_err.and_then(|db_err| db_err.code().map(|code| code == "23503")).unwrap_or(false) {
                return WorkdayError::WorkdayNotFound;
            }
            if db_err.and_then(|db_err| db_err.code().map(|code| code == "23505"))
                .unwrap_or(false)
            {
                return WorkdayError::WorkdayGarbageAlreadyExists;
            }

            error!("Failed to create workday garbage: {:?}", e);
            WorkdayError::DatabaseError
        })
    }

    #[tracing::instrument(
        name = "db.workdays.delete_workday_garbage",
        skip(self),
        fields(
            driver_id = %driver_id,
            date = %date,
        )
    )]
    async fn delete_workday_garbage(
        &self,
        driver_id: Uuid,
        date: NaiveDate,
    ) -> Result<(), WorkdayError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM workday_garbage
            WHERE workday_date = $1
            AND fk_driver_id = $2
            "#,
            date,
            driver_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to delete workday garbage: {:?}", e);
            WorkdayError::DatabaseError
        })?;

        if result.rows_affected() == 0 {
            return Err(WorkdayError::WorkdayGarbageNotFound);
        }

        Ok(())
    }

    #[tracing::instrument(
        name = "db.workdays.get_workday_document_years",
        skip(self),
        fields(
            driver_id = %driver_id,
        )
    )]
    async fn get_workday_document_years(&self, driver_id: Uuid) -> Result<Vec<i32>, WorkdayError> {
        let records = sqlx::query!(
            r#"
            SELECT DISTINCT year
            FROM workday_documents
            WHERE fk_driver_id = $1
            ORDER BY year ASC
            "#,
            driver_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to get workday document years: {:?}", e);
            WorkdayError::DatabaseError
        })?;

        Ok(records.into_iter().map(|r| r.year).collect())
    }

    #[tracing::instrument(
        name = "db.workdays.get_workday_documents_by_year",
        skip(self),
        fields(
            driver_id = %driver_id,
            year = %year,
        )
    )]
    async fn get_workday_documents_by_year(
        &self,
        driver_id: Uuid,
        year: i32,
    ) -> Result<Vec<WorkdayDocumentInformation>, WorkdayError> {
        let records = sqlx::query!(
            r#"
            SELECT wd.month, wd.year, d.created_at as "generated_at?"
            FROM workday_documents wd
            LEFT JOIN documents d ON wd.fk_document_id = d.pk_document_id
            WHERE wd.fk_driver_id = $1 AND wd.year = $2
            "#,
            driver_id,
            year
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to get workday documents by year: {:?}", e);
            WorkdayError::DatabaseError
        })?;

        Ok(records
            .into_iter()
            .map(|r| WorkdayDocumentInformation {
                month: r.month as u32,
                year: r.year as u32,
                generated_at: r.generated_at,
            })
            .collect())
    }

    #[tracing::instrument(
        name = "db.workdays.get_workday_document_record",
        skip(self),
        fields(
            db.system = "postgresql",
            db.operation = "SELECT",
            driver_id = %driver_id,
            month = %month,
            year = %year,
        )
    )]
    async fn get_workday_document_record(
        &self,
        driver_id: Uuid,
        month: i32,
        year: i32,
    ) -> Result<Option<WorkdayDocument>, WorkdayError> {
        let workday_document = sqlx::query_as::<_, WorkdayDocumentRow>(
            "SELECT fk_driver_id, month, year, fk_document_id
             FROM workday_documents
             WHERE fk_driver_id = $1 AND month = $2 AND year = $3",
        )
        .bind(driver_id)
        .bind(month)
        .bind(year)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to get workday document record: {:?}", e);
            WorkdayError::DatabaseError
        })?;

        let workday_document = match workday_document {
            Some(doc) => doc,
            None => return Ok(None),
        };

        let document = sqlx::query_as::<_, DocumentRow>(
            "SELECT pk_document_id, s3_file_path, file_name, created_at
             FROM documents
             WHERE pk_document_id = $1",
        )
        .bind(workday_document.fk_document_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            error!(
                "Failed to get document for workday document record: {:?}",
                e
            );
            WorkdayError::DatabaseError
        })?;

        Ok(Some(WorkdayDocument {
            fk_driver_id: workday_document.fk_driver_id,
            month: workday_document.month,
            year: workday_document.year,
            s3_file_path: document.s3_file_path,
            file_name: document.file_name,
            created_at: document.created_at,
        }))
    }
}
