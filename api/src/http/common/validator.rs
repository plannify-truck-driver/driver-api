use axum::{
    extract::{FromRequest, Request},
    Json,
};
use serde::de::DeserializeOwned;
use serde_yaml::{Mapping, Value};
use validator::Validate;

use crate::ApiError;

pub struct ValidatedJson<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|e| ApiError::BadRequest { error_code: "MISSING_ATTRIBUTE".to_string(), content: Some(Value::String(e.body_text())) })?;

        value.validate().map_err(|err| {
            let mut content = Mapping::new();
            let errors = err.field_errors();
            for (field, field_errors) in errors {
                let messages: Vec<String> = field_errors
                    .iter()
                    .map(|e| {
                        if let Some(message) = &e.message {
                            message.to_string()
                        } else {
                            format!("Invalid value for field '{}'", field)
                        }
                    })
                    .collect();
                content.insert(
                    Value::String(field.to_string()),
                    Value::Sequence(messages.into_iter().map(Value::String).collect()),
                );
            }

            ApiError::BadRequest { error_code: "BODY_VALIDATION".to_string(), content: Some(Value::Mapping(content)) }
        })?;

        Ok(ValidatedJson(value))
    }
}