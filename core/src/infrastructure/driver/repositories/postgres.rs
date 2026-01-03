use sqlx::PgPool;

use crate::{domain::driver::{entities::{CreateDriverRequest, DriverRow}, port::DriverRepository}, infrastructure::driver::repositories::error::DriverError};

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

    async fn update_driver(
        &self,
        driver: DriverRow,
    ) -> Result<DriverRow, DriverError> {
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
}