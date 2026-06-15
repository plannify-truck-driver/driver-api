use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    domain::{
        common::constants::EnumDriverMailType,
        driver::entities::DriverRow,
        mail::{
            entities::{DriverMailAttachmentRow, DriverMailRow, DriverMailTypeRow, MailStatus},
            port::MailDatabaseRepository,
        },
    },
    infrastructure::mail::repositories::error::MailError,
};
use tracing::error;

#[derive(sqlx::FromRow)]
struct DocumentIdRow {
    pk_document_id: Uuid,
}

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
    #[tracing::instrument(
        name = "db.mails.create_mail",
        skip(self),
        fields(
            db.system = "postgresql",
            db.operation = "INSERT",
        )
    )]
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

    #[tracing::instrument(
        name = "db.mails.update_mail_status",
        skip(self),
        fields(
            db.system = "postgresql",
            db.operation = "UPDATE",
            mail_id = %mail_id,
            status = ?status,
            sent_at = ?sent_at,
        )
    )]
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

    #[tracing::instrument(
        name = "db.mails.get_mails",
        skip(self),
        fields(
            db.system = "postgresql",
            db.operation = "SELECT",
            driver_id = %driver_id,
            page = %page,
            limit = %limit,
        )
    )]
    async fn get_mails(
        &self,
        driver_id: Uuid,
        page: u32,
        limit: u32,
    ) -> Result<(Vec<DriverMailRow>, u32), MailError> {
        let total = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM driver_mails WHERE fk_driver_id = $1"#,
            driver_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to count mails: {:?}", e);
            MailError::DatabaseError
        })?
        .unwrap_or(0) as u32;

        let mails = sqlx::query_as!(
            DriverMailRow,
            r#"
            SELECT pk_driver_mail_id, fk_driver_id, fk_employee_id, fk_mail_type_id,
                   email_used, status as "status: MailStatus", description, content, created_at, sent_at
            FROM driver_mails
            WHERE fk_driver_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            driver_id,
            limit as i64,
            ((page - 1) * limit) as i64,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to get mails: {:?}", e);
            MailError::DatabaseError
        })?;

        Ok((mails, total))
    }

    #[tracing::instrument(
        name = "db.mails.get_mail_types",
        skip(self),
        fields(db.system = "postgresql", db.operation = "SELECT")
    )]
    async fn get_mail_types(&self) -> Result<Vec<DriverMailTypeRow>, MailError> {
        sqlx::query_as!(
            DriverMailTypeRow,
            r#"SELECT pk_driver_mail_type_id, label, index, is_editable FROM driver_mail_types ORDER BY index ASC"#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to get mail types: {:?}", e);
            MailError::DatabaseError
        })
    }

    #[tracing::instrument(
        name = "db.mails.get_mail_type_by_id",
        skip(self),
        fields(db.system = "postgresql", db.operation = "SELECT", mail_type_id = %mail_type_id)
    )]
    async fn get_mail_type_by_id(&self, mail_type_id: i32) -> Result<DriverMailTypeRow, MailError> {
        sqlx::query_as!(
            DriverMailTypeRow,
            r#"SELECT pk_driver_mail_type_id, label, index, is_editable FROM driver_mail_types WHERE pk_driver_mail_type_id = $1"#,
            mail_type_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to get mail type {}: {:?}", mail_type_id, e);
            MailError::DatabaseError
        })?
        .ok_or(MailError::MailTypeNotFound)
    }

    #[tracing::instrument(
        name = "db.mails.get_driver_mail_preferences",
        skip(self),
        fields(db.system = "postgresql", db.operation = "SELECT", driver_id = %driver_id)
    )]
    async fn get_driver_mail_preferences(&self, driver_id: Uuid) -> Result<i32, MailError> {
        sqlx::query_scalar!(
            r#"SELECT mail_preferences FROM drivers WHERE pk_driver_id = $1"#,
            driver_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to get driver mail preferences: {:?}", e);
            MailError::DatabaseError
        })?
        .ok_or(MailError::Internal)
    }

    #[tracing::instrument(
        name = "db.mails.update_driver_mail_preferences",
        skip(self),
        fields(
            db.system = "postgresql",
            db.operation = "UPDATE",
            driver_id = %driver_id,
            mail_preferences = %mail_preferences,
        )
    )]
    async fn update_driver_mail_preferences(
        &self,
        driver_id: Uuid,
        mail_preferences: i32,
    ) -> Result<i32, MailError> {
        sqlx::query_scalar!(
            r#"UPDATE drivers SET mail_preferences = $2 WHERE pk_driver_id = $1 RETURNING mail_preferences"#,
            driver_id,
            mail_preferences,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to update driver mail preferences: {:?}", e);
            MailError::DatabaseError
        })?
        .ok_or(MailError::Internal)
    }

    #[tracing::instrument(
        name = "db.mails.get_mail_by_id",
        skip(self),
        fields(db.system = "postgresql", db.operation = "SELECT", mail_id = %mail_id)
    )]
    async fn get_mail_by_id(&self, mail_id: Uuid) -> Result<DriverMailRow, MailError> {
        sqlx::query_as!(
            DriverMailRow,
            r#"
            SELECT pk_driver_mail_id, fk_driver_id, fk_employee_id, fk_mail_type_id,
                   email_used, status as "status: MailStatus", description, content, created_at, sent_at
            FROM driver_mails
            WHERE pk_driver_mail_id = $1
            "#,
            mail_id,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to get mail {}: {:?}", mail_id, e);
            MailError::DatabaseError
        })?
        .ok_or(MailError::MailNotFound)
    }

    #[tracing::instrument(
        name = "db.mails.has_monthly_report_this_month",
        skip(self),
        fields(db.system = "postgresql", db.operation = "SELECT", driver_id = %driver_id)
    )]
    async fn has_monthly_report_this_month(
        &self,
        driver_id: Uuid,
        month: u32,
        year: i32,
    ) -> Result<bool, MailError> {
        let count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*)
            FROM driver_mails
            WHERE fk_driver_id = $1
            AND fk_mail_type_id = 4
            AND EXTRACT(MONTH FROM created_at)::INTEGER = $2
            AND EXTRACT(YEAR FROM created_at)::INTEGER = $3
            "#,
            driver_id,
            month as i32,
            year,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            error!(
                "Failed to check monthly report for driver {}: {:?}",
                driver_id, e
            );
            MailError::DatabaseError
        })?
        .unwrap_or(0);

        Ok(count > 0)
    }

    #[tracing::instrument(
        name = "db.mails.create_document",
        skip(self),
        fields(db.system = "postgresql", db.operation = "INSERT")
    )]
    async fn create_document(
        &self,
        s3_file_path: String,
        file_name: String,
    ) -> Result<Uuid, MailError> {
        let row = sqlx::query_as::<_, DocumentIdRow>(
            r#"
            INSERT INTO documents (s3_file_path, file_name, created_at)
            VALUES ($1, $2, NOW())
            RETURNING pk_document_id
            "#,
        )
        .bind(&s3_file_path)
        .bind(&file_name)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to create document ({}): {:?}", s3_file_path, e);
            MailError::DatabaseError
        })?;

        Ok(row.pk_document_id)
    }

    #[tracing::instrument(
        name = "db.mails.create_mail_attachment",
        skip(self),
        fields(db.system = "postgresql", db.operation = "INSERT", mail_id = %mail_id)
    )]
    async fn create_mail_attachment(
        &self,
        mail_id: Uuid,
        document_id: Uuid,
    ) -> Result<(), MailError> {
        sqlx::query(
            r#"
            INSERT INTO driver_mail_attachments (fk_driver_mail_id, fk_document_id)
            VALUES ($1, $2)
            "#,
        )
        .bind(mail_id)
        .bind(document_id)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!(
                "Failed to create mail attachment for mail {}: {:?}",
                mail_id, e
            );
            MailError::DatabaseError
        })?;

        Ok(())
    }

    #[tracing::instrument(
        name = "db.mails.get_mail_attachment_by_id",
        skip(self),
        fields(db.system = "postgresql", db.operation = "SELECT", attachment_id = %attachment_id)
    )]
    async fn get_mail_attachment_by_id(
        &self,
        attachment_id: Uuid,
    ) -> Result<DriverMailAttachmentRow, MailError> {
        sqlx::query_as!(
            DriverMailAttachmentRow,
            r#"
            SELECT
                a.pk_driver_mail_attachment_id,
                a.fk_driver_mail_id,
                d.file_name,
                d.s3_file_path,
                d.created_at
            FROM driver_mail_attachments a
            JOIN documents d ON d.pk_document_id = a.fk_document_id
            WHERE a.pk_driver_mail_attachment_id = $1
            "#,
            attachment_id,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to get mail attachment {}: {:?}", attachment_id, e);
            MailError::DatabaseError
        })?
        .ok_or(MailError::MailAttachmentNotFound)
    }

    #[tracing::instrument(
        name = "db.mails.get_mail_attachments",
        skip(self, mail_ids),
        fields(db.system = "postgresql", db.operation = "SELECT", mail_count = mail_ids.len())
    )]
    async fn get_mail_attachments(
        &self,
        mail_ids: Vec<Uuid>,
    ) -> Result<Vec<DriverMailAttachmentRow>, MailError> {
        if mail_ids.is_empty() {
            return Ok(Vec::new());
        }

        sqlx::query_as!(
            DriverMailAttachmentRow,
            r#"
            SELECT
                a.pk_driver_mail_attachment_id,
                a.fk_driver_mail_id,
                d.file_name,
                d.s3_file_path,
                d.created_at
            FROM driver_mail_attachments a
            JOIN documents d ON d.pk_document_id = a.fk_document_id
            WHERE a.fk_driver_mail_id = ANY($1)
            ORDER BY d.created_at ASC
            "#,
            &mail_ids as &[Uuid],
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to get mail attachments: {:?}", e);
            MailError::DatabaseError
        })
    }
}
