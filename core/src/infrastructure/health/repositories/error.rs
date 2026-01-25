use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum HealthError {
    #[error("A database error occurred")]
    DatabaseError,

    #[error("The database is unhealthy")]
    DatabaseUnhealthy,

    #[error("The cache is unhealthy")]
    CacheUnhealthy,
}
