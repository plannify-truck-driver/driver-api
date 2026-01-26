use sqlx::PgPool;

use crate::{
    domain::update::{entities::UpdateRow, port::UpdateDatabaseRepository},
    infrastructure::update::repositories::error::UpdateError,
};

#[derive(Clone)]
pub struct PostgresUpdateRepository {
    pool: PgPool,
}

impl PostgresUpdateRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl UpdateDatabaseRepository for PostgresUpdateRepository {
    async fn get_updates_by_version(
        &self,
        version: String,
        page: u32,
        limit: u32,
    ) -> Result<(Vec<UpdateRow>, u32), UpdateError> {
        let update_creation_time = sqlx::query!(
            r#"
            SELECT created_at
            FROM updates
            WHERE version = $1
            AND entity_type = 'DRIVER'
            LIMIT 1
            "#,
            version
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| UpdateError::DatabaseError)?;

        if update_creation_time.is_none() {
            return Err(UpdateError::NotFound);
        }

        let created_at = update_creation_time.unwrap().created_at;

        let rows = sqlx::query_as!(
            UpdateRow,
            r#"
            SELECT pk_update_id, version, description, entity_type as "entity_type: _", mandatory_completion_date, fk_created_employee_id, created_at
            FROM updates
            WHERE created_at > $1
            AND entity_type = 'DRIVER'
            ORDER BY created_at ASC
            OFFSET $2
            LIMIT $3
            "#,
            created_at,
            ((page - 1) * limit) as i64,
            limit as i64
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|_| UpdateError::DatabaseError)?;

        let count_row = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM updates
            WHERE created_at > $1
            AND entity_type = 'DRIVER'
            "#,
            created_at,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| UpdateError::DatabaseError)?;

        let total_count = count_row.count.unwrap_or(0) as u32;

        Ok((rows, total_count))
    }
}
