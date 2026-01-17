use chrono::{DateTime, NaiveDate, Utc};
use sqlx::PgPool;
use tracing::error;
use uuid::Uuid;

use crate::{
    domain::workday::{
        entities::{CreateWorkdayRequest, UpdateWorkdayRequest, WorkdayGarbageRow, WorkdayRow},
        port::WorkdayRepository,
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

impl WorkdayRepository for PostgresWorkdayRepository {
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

    async fn get_workday_documents(&self, driver_id: Uuid) -> Result<Vec<i32>, WorkdayError> {
        let records = sqlx::query!(
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
            driver_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to get workday documents: {:?}", e);
            WorkdayError::DatabaseError
        })?;

        Ok(records.into_iter().filter_map(|r| r.year).collect())
    }

    async fn get_workday_documents_by_year(
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
            error!("Failed to get workday documents by year: {:?}", e);
            WorkdayError::DatabaseError
        })?;

        Ok(records.into_iter().filter_map(|r| r.month).collect())
    }
}
