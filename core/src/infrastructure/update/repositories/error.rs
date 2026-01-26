use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum UpdateError {
    #[error("A database error occurred")]
    DatabaseError,

    #[error("An internal error occurred")]
    Internal,

    #[error("The requested update was not found")]
    NotFound,
}
