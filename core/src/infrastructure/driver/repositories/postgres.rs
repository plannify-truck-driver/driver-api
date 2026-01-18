use sqlx::PgPool;

use crate::{
    domain::driver::{
        entities::{
            CreateDriverRequest, DriverLimitationRow, DriverRow, DriverSuspensionRow, EntityType,
        },
        port::DriverRepository,
    },
    infrastructure::driver::repositories::error::DriverError,
};
use tracing::error;

#[derive(Clone)]
pub struct PostgresDriverRepository {
    pool: PgPool,
}

impl PostgresDriverRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl DriverRepository for PostgresDriverRepository {
    async fn get_number_of_drivers(&self) -> Result<i64, DriverError> {
        sqlx::query!(
            r#"
            SELECT COUNT(*) as "count!"
            FROM drivers
            "#
        )
        .fetch_one(&self.pool)
        .await
        .map(|row| row.count)
        .map_err(|e| {
            error!("Failed to get number of drivers: {:?}", e);
            DriverError::DatabaseError
        })
    }

    async fn get_driver_by_email(&self, email: String) -> Result<DriverRow, DriverError> {
        sqlx::query_as!(
            DriverRow,
            r#"
            SELECT *
            FROM drivers
            WHERE email = $1
            "#,
            email,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| DriverError::DriverNotFound)
    }

    async fn create_driver(
        &self,
        create_request: CreateDriverRequest,
    ) -> Result<DriverRow, DriverError> {
        sqlx::query_as!(
            DriverRow,
            r#"
            INSERT INTO drivers (firstname, lastname, gender, email, password_hash, language)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
            create_request.firstname,
            create_request.lastname,
            create_request.gender,
            create_request.email,
            create_request.password,
            create_request.language,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| DriverError::DriverAlreadyExists)
    }

    async fn update_driver(&self, driver: DriverRow) -> Result<DriverRow, DriverError> {
        sqlx::query_as!(
            DriverRow,
            r#"
            UPDATE drivers
            SET firstname = $1,
                lastname = $2,
                gender = $3,
                email = $4,
                password_hash = $5,
                phone_number = $6,
                is_searchable = $7,
                allow_request_professional_agreement = $8,
                language = $9,
                rest_json = $10,
                mail_preferences = $11,
                verified_at = $12,
                last_login_at = $13,
                deactivated_at = $14,
                refresh_token_hash = $15
            WHERE pk_driver_id = $16
            RETURNING *
            "#,
            driver.firstname,
            driver.lastname,
            driver.gender,
            driver.email,
            driver.password_hash,
            driver.phone_number,
            driver.is_searchable,
            driver.allow_request_professional_agreement,
            driver.language,
            driver.rest_json,
            driver.mail_preferences,
            driver.verified_at,
            driver.last_login_at,
            driver.deactivated_at,
            driver.refresh_token_hash,
            driver.pk_driver_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| DriverError::DriverNotFound)
    }

    async fn delete_driver(&self, driver_id: uuid::Uuid) -> Result<(), DriverError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM drivers
            WHERE pk_driver_id = $1
            "#,
            driver_id,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            let db_err = e.as_database_error();

            if db_err
                .and_then(|db_err| db_err.code().map(|code| code == "23503"))
                .unwrap_or(false)
            {
                DriverError::DriverNotFound
            } else {
                error!("Failed to delete driver {}: {:?}", driver_id, e);
                DriverError::DatabaseError
            }
        })?;

        if result.rows_affected() == 0 {
            Err(DriverError::DriverNotFound)
        } else {
            Ok(())
        }
    }

    async fn get_actual_driver_limitation(
        &self,
    ) -> Result<Option<DriverLimitationRow>, DriverError> {
        error!("Fetching actual driver limitation");

        sqlx::query_as!(
            DriverLimitationRow,
            r#"
            SELECT pk_maximum_entity_limit_id, entity_type as "entity_type: _", maximum_limit, fk_created_employee_id, start_at, end_at, created_at
            FROM maximum_entity_limits
            WHERE entity_type::text = 'DRIVER'
            AND start_at <= $1
            AND (end_at IS NULL OR end_at > $1)
            ORDER BY start_at DESC
            LIMIT 1
            "#,
            chrono::Utc::now(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to get actual driver limitation: {:?}", e);
            DriverError::DatabaseError
        })
    }

    async fn create_driver_limitation(
        &self,
        limitation: DriverLimitationRow,
    ) -> Result<DriverLimitationRow, DriverError> {
        error!("Creating driver limitation");

        sqlx::query_as!(
            DriverLimitationRow,
            r#"
            INSERT INTO maximum_entity_limits (entity_type, maximum_limit, fk_created_employee_id, start_at, end_at, created_at)
            VALUES ($1::entity_type, $2, $3, $4, $5, $6)
            RETURNING pk_maximum_entity_limit_id, entity_type as "entity_type: EntityType", maximum_limit, fk_created_employee_id, start_at, end_at, created_at
            "#,
            limitation.entity_type as EntityType,
            limitation.maximum_limit,
            limitation.fk_created_employee_id,
            limitation.start_at,
            limitation.end_at,
            limitation.created_at,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            eprintln!("Error creating driver limitation: {:?}", e);
            error!("Failed to create driver limitation: {:?}", e);
            DriverError::DatabaseError
        })
    }

    async fn delete_driver_limitation(&self, limitation_id: i32) -> Result<(), DriverError> {
        error!("Deleting driver limitation");

        let result = sqlx::query!(
            r#"
            DELETE FROM maximum_entity_limits
            WHERE pk_maximum_entity_limit_id = $1
            "#,
            limitation_id,
        )
        .execute(&self.pool)
        .await
        .map_err(|_| DriverError::DatabaseError)?;

        if result.rows_affected() == 0 {
            Err(DriverError::DriverLimitationNotFound)
        } else {
            Ok(())
        }
    }

    async fn get_current_driver_suspension(
        &self,
        driver_id: uuid::Uuid,
    ) -> Result<Option<DriverSuspensionRow>, DriverError> {
        sqlx::query_as!(
            DriverSuspensionRow,
            r#"
            SELECT pk_driver_suspension_id, fk_driver_id, fk_created_employee_id, can_access_restricted_space, driver_message, title, description, start_at, end_at, created_at
            FROM driver_suspensions
            WHERE fk_driver_id = $1
            AND start_at <= $2
            AND (end_at IS NULL OR end_at > $2)
            LIMIT 1
            "#,
            driver_id,
            chrono::Utc::now(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to get current driver suspension: {:?}", e);
            DriverError::DatabaseError
        })
    }

    async fn create_driver_suspension(
        &self,
        suspension: DriverSuspensionRow,
    ) -> Result<DriverSuspensionRow, DriverError> {
        error!("Creating driver suspension");
        sqlx::query_as!(
            DriverSuspensionRow,
            r#"
            INSERT INTO driver_suspensions (fk_driver_id, fk_created_employee_id, can_access_restricted_space, driver_message, title, description, start_at, end_at, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING pk_driver_suspension_id, fk_driver_id, fk_created_employee_id, can_access_restricted_space, driver_message, title, description, start_at, end_at, created_at
            "#,
            suspension.fk_driver_id,
            suspension.fk_created_employee_id,
            suspension.can_access_restricted_space,
            suspension.driver_message,
            suspension.title,
            suspension.description,
            suspension.start_at,
            suspension.end_at,
            suspension.created_at,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to create driver suspension: {:?}", e);
            DriverError::DatabaseError
        })
    }

    async fn delete_driver_suspension(&self, suspension_id: i32) -> Result<(), DriverError> {
        error!("Deleting driver suspension with ID: {}", suspension_id);
        let result = sqlx::query!(
            r#"
            DELETE FROM driver_suspensions
            WHERE pk_driver_suspension_id = $1
            "#,
            suspension_id,
        )
        .execute(&self.pool)
        .await
        .map_err(|_| DriverError::DatabaseError)?;

        if result.rows_affected() == 0 {
            Err(DriverError::DriverSuspensionNotFound)
        } else {
            Ok(())
        }
    }
}
