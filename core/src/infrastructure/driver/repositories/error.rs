use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum DriverError {
    #[error("A database error occurred")]
    DatabaseError,

    #[error("An internal error occurred")]
    Internal,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Driver already exists")]
    DriverAlreadyExists,

    #[error("Email domain '{domain}' is denylisted")]
    EmailDomainDenylisted { domain: String },

    #[error("Driver not found")]
    DriverNotFound,

    #[error("Driver limit reached")]
    DriverLimitReached {
        start_at: String,
        end_at: Option<String>,
    },

    #[error("Driver limitation not found")]
    DriverLimitationNotFound,
}
