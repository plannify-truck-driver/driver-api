use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    domain::{
        common::constants::EnumDriverMailType,
        driver::entities::DriverRow,
        mail::{
            entities::{DriverMailRow, MailStatus},
            port::MailDatabaseRepository,
        },
    },
    infrastructure::mail::repositories::error::MailError,
};
use tracing::error;

#[derive(Clone)]
pub struct PostgresMailRepository {
    pool: PgPool,
}

impl PostgresMailRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl MailDatabaseRepository for PostgresMailRepository {
    async fn create_mail(
        &self,
        driver: DriverRow,
        mail_type: EnumDriverMailType,
        description: String,
        content: Option<String>,
    ) -> Result<DriverMailRow, MailError> {
        sqlx::query_as!(
            DriverMailRow,
            r#"
            INSERT INTO driver_mails (fk_driver_id, fk_employee_id, fk_mail_type_id, email_used, status, description, content, created_at)
            VALUES ($1, NULL, $2, $3, $4, $5, $6, NOW())
            RETURNING pk_driver_mail_id, fk_driver_id, fk_employee_id, fk_mail_type_id, email_used, status as "status: MailStatus", description, content, created_at, sent_at
            "#,
            driver.pk_driver_id,
            mail_type.as_id(),
            driver.email.clone(),
            MailStatus::PENDING as MailStatus,
            description,
            content,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if e.as_database_error()
                .and_then(|db_err| db_err.code().map(|code| code == "23503"))
                .unwrap_or(false)
            {
                return MailError::MailTypeNotFound;
            }

            error!("Failed to create mail: {:?}", e);
            MailError::DatabaseError
        })
    }

    async fn update_mail_status(
        &self,
        mail_id: Uuid,
        status: MailStatus,
        sent_at: Option<DateTime<Utc>>,
    ) -> Result<DriverMailRow, MailError> {
        sqlx::query_as!(
            DriverMailRow,
            r#"
            UPDATE driver_mails
            SET status = $2, sent_at = $3
            WHERE pk_driver_mail_id = $1
            RETURNING pk_driver_mail_id, fk_driver_id, fk_employee_id, fk_mail_type_id, email_used, status as "status: MailStatus", description, content, created_at, sent_at
            "#,
            mail_id,
            status as MailStatus,
            sent_at,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if e.as_database_error()
                .and_then(|db_err| db_err.code().map(|code| code == "23503"))
                .unwrap_or(false)
            {
                return MailError::MailNotFound;
            }

            error!("Failed to update mail status: {:?}", e);
            MailError::DatabaseError
        })
    }
}
