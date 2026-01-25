use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct DriverMailTypeRow {
    pub pk_driver_mail_type_id: i32,
    pub label: String,
    pub index: i32,    
    pub is_editable: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DriverMailType {
    pub pk_driver_mail_type_id: i32,
    pub label: String,
    pub is_editable: bool,
}

impl DriverMailTypeRow {
    pub fn to_driver_mail_type(&self) -> DriverMailType {
        DriverMailType {
            pk_driver_mail_type_id: self.pk_driver_mail_type_id,
            label: self.label.clone(),
            is_editable: self.is_editable,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, sqlx::Type)]
#[sqlx(type_name = "mail_status")]
pub enum MailStatus {
    PENDING,
    SUCCESS,
    FAILED,
}

impl FromStr for MailStatus {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "PENDING" => Ok(MailStatus::PENDING),
            "SUCCESS" => Ok(MailStatus::SUCCESS),
            "FAILED" => Ok(MailStatus::FAILED),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct DriverMailRow {
    pub pk_driver_mail_id: Uuid,
    pub fk_driver_id: Uuid,
    pub fk_employee_id: Option<Uuid>,
    pub fk_mail_type_id: i32,
    pub email_used: String,
    pub status: MailStatus,
    pub description: String,
    pub content: Option<String>,
    pub created_at: DateTime<Utc>,
    pub sent_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DriverMail {
    pub pk_driver_mail_id: Uuid,
    pub mail_type: DriverMailType,
    pub email_used: String,
    pub status: MailStatus,
    pub description: String,
    pub content: Option<String>,
    pub created_at: DateTime<Utc>,
    pub sent_at: Option<DateTime<Utc>>,
}

impl DriverMailRow {
    pub fn to_driver_mail(&self, mail_type: DriverMailType) -> DriverMail {
        DriverMail {
            pk_driver_mail_id: self.pk_driver_mail_id,
            mail_type,
            email_used: self.email_used.clone(),
            status: self.status.clone(),
            description: self.description.clone(),
            content: self.content.clone(),
            created_at: self.created_at,
            sent_at: self.sent_at,
        }
    }
}
