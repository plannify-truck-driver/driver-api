use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum HealthError {
    #[error("A database error occurred")]
    DatabaseError,
}
