use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum WorkdayError {
    #[error("A database error occurred")]
    DatabaseError,

    #[error("Workday already exists for the given date")]
    WorkdayAlreadyExists,

    #[error("Workday not found")]
    WorkdayNotFound,
}
