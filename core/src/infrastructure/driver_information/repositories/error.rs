use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum DriverInformationError {
    #[error("Internal server error")]
    Internal,

    #[error("Database error")]
    DatabaseError,
}
