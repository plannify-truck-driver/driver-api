use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use plannify_driver_api_core::{
    domain::common::CoreError,
    infrastructure::{
        driver::repositories::error::DriverError, health::repositories::error::HealthError,
        mail::repositories::error::MailError, workday::repositories::error::WorkdayError,
    },
};
use serde::{Deserialize, Serialize};
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
    Forbidden {
        error_code: String,
        content: Option<Value>,
    },

    #[error("Not found")]
    NotFound { error_code: String },

    #[error("Bad request")]
    BadRequest {
        error_code: String,
        content: Option<Value>,
    },

    #[error("Conflict")]
    Conflict { error_code: String },
}

impl ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::StartupError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::ServiceUnavailable { .. } => StatusCode::SERVICE_UNAVAILABLE,
            ApiError::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Unauthorized { .. } => StatusCode::UNAUTHORIZED,
            ApiError::Forbidden { .. } => StatusCode::FORBIDDEN,
            ApiError::NotFound { .. } => StatusCode::NOT_FOUND,
            ApiError::BadRequest { .. } => StatusCode::BAD_REQUEST,
            ApiError::Conflict { .. } => StatusCode::CONFLICT,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ErrorBody {
    pub message: String,
    pub error_code: String,
    pub content: Option<Value>,
    pub status: u16,
}

impl From<ApiError> for ErrorBody {
    fn from(val: ApiError) -> Self {
        let status = val.status_code().as_u16();
        let message = val.to_string();
        match val {
            ApiError::Unauthorized { error_code } => ErrorBody {
                message,
                error_code,
                content: None,
                status,
            },
            ApiError::Forbidden {
                error_code,
                content,
            } => ErrorBody {
                message,
                error_code,
                content,
                status,
            },
            ApiError::NotFound { error_code } => ErrorBody {
                message,
                error_code,
                content: None,
                status,
            },
            ApiError::BadRequest {
                error_code,
                content,
            } => ErrorBody {
                message,
                error_code,
                content,
                status,
            },
            ApiError::Conflict { error_code } => ErrorBody {
                message,
                error_code,
                content: None,
                status,
            },
            _ => ErrorBody {
                message,
                error_code: String::new(),
                content: None,
                status,
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
            CoreError::CorsBindingError { message } => ApiError::StartupError {
                msg: format!("CORS binding error: {}", message),
            },
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
            DriverError::Internal => ApiError::InternalServerError,
            DriverError::InvalidCredentials => ApiError::Unauthorized {
                error_code: "INVALID_CREDENTIALS".to_string(),
            },
            DriverError::DriverAlreadyExists => ApiError::Conflict {
                error_code: "DRIVER_ALREADY_EXISTS".to_string(),
            },
            DriverError::EmailDomainDenylisted { domain } => {
                let mut content = Mapping::new();
                content.insert(Value::String("domain".to_string()), Value::String(domain));
                ApiError::BadRequest {
                    error_code: "EMAIL_DOMAIN_DENYLISTED".to_string(),
                    content: Some(Value::Mapping(content)),
                }
            }
            DriverError::DriverNotFound => ApiError::NotFound {
                error_code: "DRIVER_NOT_FOUND".to_string(),
            },
            DriverError::DriverLimitReached { start_at, end_at } => {
                let mut content = Mapping::new();
                content.insert(
                    Value::String("start_at".to_string()),
                    Value::String(start_at),
                );
                if let Some(end_at) = end_at {
                    content.insert(Value::String("end_at".to_string()), Value::String(end_at));
                }
                ApiError::BadRequest {
                    error_code: "DRIVER_LIMIT_REACHED".to_string(),
                    content: Some(Value::Mapping(content)),
                }
            }
            DriverError::DriverLimitationNotFound => ApiError::NotFound {
                error_code: "DRIVER_LIMITATION_NOT_FOUND".to_string(),
            },
            DriverError::DriverSuspensionNotFound => ApiError::NotFound {
                error_code: "DRIVER_SUSPENSION_NOT_FOUND".to_string(),
            },
            DriverError::DriverSuspension {
                message,
                start_at,
                end_at,
            } => {
                let mut content = Mapping::new();
                if let Some(msg) = message {
                    content.insert(Value::String("message".to_string()), Value::String(msg));
                }
                content.insert(
                    Value::String("start_at".to_string()),
                    Value::String(start_at),
                );
                if let Some(end_at) = end_at {
                    content.insert(Value::String("end_at".to_string()), Value::String(end_at));
                }
                ApiError::Forbidden {
                    error_code: "DRIVER_SUSPENDED".to_string(),
                    content: Some(Value::Mapping(content)),
                }
            }
            DriverError::InvalidRestPeriod { details } => {
                let mut content = Mapping::new();
                content.insert(Value::String("details".to_string()), Value::String(details));
                ApiError::BadRequest {
                    error_code: "INVALID_REST_PERIOD".to_string(),
                    content: Some(Value::Mapping(content)),
                }
            }
            DriverError::EmailSendError => ApiError::InternalServerError,
        }
    }
}

impl From<WorkdayError> for ApiError {
    fn from(error: WorkdayError) -> Self {
        match error {
            WorkdayError::DatabaseError => ApiError::InternalServerError,
            WorkdayError::WorkdayAlreadyExists => ApiError::Conflict {
                error_code: "WORKDAY_ALREADY_EXISTS".to_string(),
            },
            WorkdayError::WorkdayNotFound => ApiError::NotFound {
                error_code: "WORKDAY_NOT_FOUND".to_string(),
            },
            WorkdayError::WorkdayGarbageAlreadyExists => ApiError::Conflict {
                error_code: "WORKDAY_GARBAGE_ALREADY_EXISTS".to_string(),
            },
            WorkdayError::WorkdayGarbageNotFound => ApiError::NotFound {
                error_code: "WORKDAY_GARBAGE_NOT_FOUND".to_string(),
            },
        }
    }
}

impl From<MailError> for ApiError {
    fn from(_error: MailError) -> Self {
        match _error {
            MailError::Internal => ApiError::InternalServerError,
            MailError::CannotCreateMessage => ApiError::InternalServerError,
            MailError::CannotSendMessage => ApiError::InternalServerError,
            MailError::DatabaseError => ApiError::InternalServerError,
            MailError::MailNotFound => ApiError::NotFound {
                error_code: "MAIL_NOT_FOUND".to_string(),
            },
            MailError::MailTypeNotFound => ApiError::NotFound {
                error_code: "MAIL_TYPE_NOT_FOUND".to_string(),
            },
        }
    }
}
