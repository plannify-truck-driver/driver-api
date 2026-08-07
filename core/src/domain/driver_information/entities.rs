use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::prelude::FromRow;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq, Clone, sqlx::Type)]
#[sqlx(type_name = "driver_informations_type")]
pub enum DriverInformationType {
    INFO,
    WARNING,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct DriverInformationRow {
    pub pk_information_id: i64,
    pub information_type: DriverInformationType,
    pub message: Value,
    pub created_at: DateTime<Utc>,
    pub show_at: DateTime<Utc>,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct DriverInformation {
    #[serde(rename = "type")]
    pub information_type: DriverInformationType,
    pub message: Value,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DriverInformationCache {
    pub information_type: DriverInformationType,
    pub message: Value,
    pub show_at: DateTime<Utc>,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
}

impl DriverInformationRow {
    pub fn to_cache(&self) -> DriverInformationCache {
        DriverInformationCache {
            information_type: self.information_type.clone(),
            message: self.message.clone(),
            show_at: self.show_at,
            start_at: self.start_at,
            end_at: self.end_at,
        }
    }
}

impl DriverInformationCache {
    pub fn to_driver_information(&self) -> DriverInformation {
        DriverInformation {
            information_type: self.information_type.clone(),
            message: self.message.clone(),
            start_at: self.start_at,
            end_at: self.end_at,
        }
    }
}
