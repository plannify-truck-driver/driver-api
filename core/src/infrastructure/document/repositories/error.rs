use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum DocumentError {
    #[error("An internal error occurred")]
    Internal,
}
