use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum EmployeeError {
    #[error("A database error occurred")]
    DatabaseError,
}
