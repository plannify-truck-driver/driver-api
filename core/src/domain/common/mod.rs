use thiserror::Error;

pub mod services;

#[derive(Error, Debug, Clone)]
pub enum CoreError {
    #[error("Service is currently unavailable")]
    ServiceUnavailable(String),

    #[error("Unable to bind CORS origins: {message}")]
    CorsBindingError { message: String },
}
