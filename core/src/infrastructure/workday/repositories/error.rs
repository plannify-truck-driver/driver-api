use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum WorkdayError {
    #[error("A database error occurred")]
    DatabaseError,

    #[error("Workday already exists for the given date")]
    WorkdayAlreadyExists,

    #[error("Workday not found")]
    WorkdayNotFound,

    #[error("Workday garbage already exists for the given date")]
    WorkdayGarbageAlreadyExists,

    #[error("Workday garbage not found")]
    WorkdayGarbageNotFound,
}
