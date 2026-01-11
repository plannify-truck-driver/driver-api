use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum WorkdayError {
    #[error("A database error occurred")]
    DatabaseError,
}
