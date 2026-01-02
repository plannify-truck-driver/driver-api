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
}