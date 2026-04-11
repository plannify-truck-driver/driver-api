use aws_sdk_s3::Client as S3Client;
use redis::{AsyncCommands, aio::ConnectionManager};
use sqlx::PgPool;

use crate::domain::health::{entities::IsHealthy, port::HealthRepository};

#[derive(Clone)]
pub struct PostgresHealthRepository {
    pub(crate) pool: PgPool,
    pub(crate) cache: ConnectionManager,
    pub(crate) s3_client: S3Client,
    pub(crate) s3_bucket: String,
}

impl PostgresHealthRepository {
    pub fn new(
        pool: PgPool,
        cache: ConnectionManager,
        s3_client: S3Client,
        s3_bucket: String,
    ) -> Self {
        Self { pool, cache, s3_client, s3_bucket }
    }
}

impl HealthRepository for PostgresHealthRepository {
    async fn ping(&self) -> IsHealthy {
        let mut conn = self.cache.clone();

        let database = sqlx::query!("SELECT 1 as health_check")
            .fetch_one(&self.pool)
            .await
            .is_ok();

        let cache = conn.ping::<()>().await.is_ok();

        let storage = self
            .s3_client
            .head_bucket()
            .bucket(&self.s3_bucket)
            .send()
            .await
            .is_ok();

        IsHealthy::new(database, cache, storage)
    }
}
