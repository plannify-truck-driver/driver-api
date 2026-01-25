use redis::{AsyncCommands, aio::ConnectionManager};
use sqlx::PgPool;

use crate::domain::health::{entities::IsHealthy, port::HealthRepository};

#[derive(Clone)]
pub struct PostgresHealthRepository {
    pub(crate) pool: PgPool,
    pub(crate) cache: ConnectionManager,
}

impl PostgresHealthRepository {
    pub fn new(pool: PgPool, cache: ConnectionManager) -> Self {
        Self { pool, cache }
    }
}

impl HealthRepository for PostgresHealthRepository {
    async fn ping(&self) -> IsHealthy {
        let mut conn = self.cache.clone();

        IsHealthy::new(
            sqlx::query!("SELECT 1 as health_check")
                .fetch_one(&self.pool)
                .await
                .is_ok(),
            conn.ping::<()>().await.is_ok(),
        )
    }
}
