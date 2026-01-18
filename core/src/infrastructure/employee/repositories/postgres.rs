use sqlx::PgPool;
use tracing::error;

use crate::{
    domain::employee::{entities::EmployeeRow, port::EmployeeRepository},
    infrastructure::employee::repositories::error::EmployeeError,
};

#[derive(Clone)]
pub struct PostgresEmployeeRepository {
    pool: PgPool,
}

impl PostgresEmployeeRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl EmployeeRepository for PostgresEmployeeRepository {
    async fn get_first_employee(&self) -> Result<Option<EmployeeRow>, EmployeeError> {
        error!("Function get_first_employee called");

        let result = sqlx::query_as!(
            EmployeeRow,
            r#"
            SELECT *
            FROM employees
            ORDER BY created_at ASC
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to get first employee: {:?}", e);
            EmployeeError::DatabaseError
        });

        if let Ok(employees) = result {
            Ok(employees.into_iter().next())
        } else {
            Err(EmployeeError::DatabaseError)
        }
    }
}
