use sqlx::PgPool;
use tracing::error;

use crate::{
    domain::administrator::{entities::AdministratorRow, port::AdministratorRepository},
    infrastructure::administrator::repositories::error::AdministratorError,
};

#[derive(Clone)]
pub struct PostgresAdministratorRepository {
    pool: PgPool,
}

impl PostgresAdministratorRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl AdministratorRepository for PostgresAdministratorRepository {
    #[tracing::instrument(
        name = "db.administrators.get_first_administrator",
        skip(self),
        fields(
            db.system = "postgresql",
            db.operation = "SELECT",
        )
    )]
    async fn get_first_administrator(
        &self,
    ) -> Result<Option<AdministratorRow>, AdministratorError> {
        error!("Function get_first_administrator called");

        let result = sqlx::query_as!(
            AdministratorRow,
            r#"
            SELECT *
            FROM administrators
            ORDER BY created_at ASC
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to get first administrator: {:?}", e);
            AdministratorError::DatabaseError
        });

        if let Ok(administrators) = result {
            Ok(administrators.into_iter().next())
        } else {
            Err(AdministratorError::DatabaseError)
        }
    }
}
