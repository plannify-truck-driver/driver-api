use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use plannify_driver_api_core::{
    domain::common::CoreError, infrastructure::{driver::repositories::error::DriverError, health::repositories::error::HealthError}
};
use serde::Serialize;
use serde_yaml::{Mapping, Value};
use thiserror::Error;
use utoipa::ToSchema;

/// Unified error type for HTTP API responses
#[derive(Debug, Error, Clone)]
pub enum ApiError {
    #[error("Service is unavailable: {msg}")]
    ServiceUnavailable { msg: String },

    #[error("Internal server error")]
    InternalServerError,

    #[error("Startup error: {msg}")]
    StartupError { msg: String },

    #[error("Unauthorized access")]
    Unauthorized { error_code: String },

    #[error("Forbidden")]
    Forbidden { error_code: String },

    #[error("Not found")]
    NotFound { error_code: String },

    #[error("Bad request")]
    BadRequest { error_code: String, content: Option<Value> },

    #[error("Conflict")]
    Conflict { error_code: String },
}

impl ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::StartupError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::ServiceUnavailable { .. } => StatusCode::SERVICE_UNAVAILABLE,
            ApiError::InternalServerError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Unauthorized { .. } => StatusCode::UNAUTHORIZED,
            ApiError::Forbidden { .. } => StatusCode::FORBIDDEN,
            ApiError::NotFound { .. } => StatusCode::NOT_FOUND,
            ApiError::BadRequest { .. } => StatusCode::BAD_REQUEST,
            ApiError::Conflict { .. } => StatusCode::CONFLICT,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorBody {
    pub message: String,
    pub error_code: String,
    pub content: Option<Value>,
    pub status: u16,
}

impl Into<ErrorBody> for ApiError {
    fn into(self) -> ErrorBody {
        let status = self.status_code().as_u16();
        let message = self.to_string();
        match self {
            ApiError::Unauthorized { error_code } => ErrorBody {
                message: message,
                error_code: error_code,
                content: None,
                status: status,
            },
            ApiError::Forbidden { error_code } => ErrorBody {
                message: message,
                error_code: error_code,
                content: None,
                status: status,
            },
            ApiError::NotFound { error_code } => ErrorBody {
                message: message,
                error_code: error_code,
                content: None,
                status: status,
            },
            ApiError::BadRequest { error_code, content } => ErrorBody {
                message: message,
                error_code: error_code,
                content: content,
                status: status,
            },
            ApiError::Conflict { error_code } => ErrorBody {
                message: message,
                error_code: error_code,
                content: None,
                status: status,
            },
            _ => ErrorBody {
                message: message,
                error_code: String::new(),
                content: None,
                status: status,
            },
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status_code(), Json::<ErrorBody>(self.into())).into_response()
    }
}

impl From<CoreError> for ApiError {
    fn from(error: CoreError) -> Self {
        match error {
            CoreError::ServiceUnavailable(message) => ApiError::ServiceUnavailable { msg: message },
            CoreError::CorsBindingError { message } => ApiError::StartupError { msg: format!("CORS binding error: {}", message) },
        }
    }
}

impl From<HealthError> for ApiError {
    fn from(error: HealthError) -> Self {
        match error {
            HealthError::DatabaseError => ApiError::InternalServerError,
        }
    }
}

impl From<DriverError> for ApiError {
    fn from(error: DriverError) -> Self {
        match error {
            DriverError::DatabaseError => ApiError::InternalServerError,
            DriverError::DriverAlreadyExists => ApiError::Conflict { error_code: "DRIVER_ALREADY_EXISTS".to_string() },
            DriverError::EmailDomainDenylisted { domain } => {
                let mut content = Mapping::new();
                content.insert(
                    Value::String("domain".to_string()),
                    Value::String(domain)
                );
                ApiError::BadRequest { 
                    error_code: "EMAIL_DOMAIN_DENYLISTED".to_string(), 
                    content: Some(Value::Mapping(content))
                }
            },
        }
    }
}