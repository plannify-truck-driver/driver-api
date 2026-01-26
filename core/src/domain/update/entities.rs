use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;
use validator::Validate;

use crate::domain::driver::entities::EntityType;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct UpdateRow {
    pub pk_update_id: i32,
    pub version: String,
    pub description: String,
    pub entity_type: EntityType,
    pub mandatory_completion_date: Option<DateTime<Utc>>,
    pub fk_created_employee_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Update {
    pub version: String,
    pub description: String,
    pub mandatory_completion_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateCache {
    pub version: String,
    pub description: String,
    pub mandatory_completion_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl UpdateRow {
    pub fn to_cache(&self) -> UpdateCache {
        UpdateCache {
            version: self.version.clone(),
            description: self.description.clone(),
            mandatory_completion_date: self.mandatory_completion_date,
            created_at: self.created_at,
        }
    }
}

impl UpdateCache {
    pub fn to_update(&self) -> Update {
        Update {
            version: self.version.clone(),
            description: self.description.clone(),
            mandatory_completion_date: self.mandatory_completion_date,
        }
    }
}

#[derive(Deserialize, Validate, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct GetUpdatesByVersionParams {
    #[validate(length(min = 1, message = "version must not be empty"))]
    pub version: String,

    #[validate(range(min = 1, message = "page must be at least 1"))]
    pub page: u32,

    #[validate(range(min = 1, max = 100, message = "limit must be between 1 and 100"))]
    pub limit: u32,
}
