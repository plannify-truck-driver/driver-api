use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum DriverError {
    #[error("A database error occurred")]
    DatabaseError,

    #[error("Driver already exists")]
    DriverAlreadyExists,

    #[error("Email domain '{domain}' is denylisted")]
    EmailDomainDenylisted { domain: String },
}