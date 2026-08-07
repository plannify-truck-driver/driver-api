use sqlx::PgPool;
use tracing::error;

use crate::{
    domain::driver_information::{
        entities::DriverInformationRow, port::DriverInformationDatabaseRepository,
    },
    infrastructure::driver_information::repositories::error::DriverInformationError,
};

#[derive(Clone)]
pub struct PostgresDriverInformationRepository {
    pool: PgPool,
}

impl PostgresDriverInformationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl DriverInformationDatabaseRepository for PostgresDriverInformationRepository {
    #[tracing::instrument(
        name = "db.driver_informations.get_driver_informations",
        skip(self),
        fields(db.system = "postgresql", db.operation = "SELECT")
    )]
    async fn get_driver_informations(
        &self,
    ) -> Result<Vec<DriverInformationRow>, DriverInformationError> {
        sqlx::query_as!(
            DriverInformationRow,
            r#"
            SELECT pk_information_id, type as "information_type: _", message, created_at, show_at, start_at, end_at
            FROM driver_informations
            WHERE end_at > NOW()
            ORDER BY start_at ASC
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to get driver informations: {:?}", e);
            DriverInformationError::DatabaseError
        })
    }
}
