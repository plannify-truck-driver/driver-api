use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct AdministratorRow {
    pub pk_administrator_id: Uuid,
    pub firstname: String,
    pub lastname: String,
    pub gender: Option<String>,
    pub personal_email: String,
    pub login_password_hash: String,
    pub created_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub deactivated_at: Option<DateTime<Utc>>,
    pub language: String,
}
