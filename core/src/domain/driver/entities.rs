use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::prelude::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct DriverRow {
    pub pk_driver_id: Uuid,
    pub firstname: String,
    pub lastname: String,
    pub gender: Option<String>,
    pub email: String,
    pub password_hash: String,
    pub phone_number: Option<String>,
    pub is_searchable: bool,
    pub allow_request_professional_agreement: bool,
    pub language: String,
    pub rest_json: Option<Value>,
    pub mail_preferences: i32,
    pub created_at: DateTime<Utc>,
    pub verified_at: Option<DateTime<Utc>>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub deactivated_at: Option<DateTime<Utc>>,
    pub refresh_token_hash: Option<String>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateDriverRequest {
    #[validate(length(
        min = 1,
        max = 255,
        message = "firstname is required and cannot be longer than 255 characters"
    ))]
    pub firstname: String,

    #[validate(length(
        min = 1,
        max = 255,
        message = "lastname is required and cannot be longer than 255 characters"
    ))]
    pub lastname: String,

    #[validate(length(equal = 1, message = "gender must be 'M', 'F' or undefined"))]
    pub gender: Option<String>,

    #[validate(email(message = "Invalid email format"))]
    #[validate(length(
        min = 1,
        max = 255,
        message = "email is required and cannot be longer than 255 characters"
    ))]
    pub email: String,

    #[validate(length(
        min = 8,
        max = 40,
        message = "password must contain at least 8 characters and at most 40 characters"
    ))]
    pub password: String,

    #[validate(length(
        equal = 2,
        message = "language must be a 2 characters code (ex: fr, en)"
    ))]
    pub language: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CreateDriverResponse {
    pub access_token: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct LoginDriverRequest {
    #[validate(email(message = "Invalid email format"))]
    #[validate(length(max = 255, message = "email cannot be longer than 255 characters"))]
    pub email: String,

    #[validate(length(min = 1, message = "password must be provided"))]
    pub password: String,
}
