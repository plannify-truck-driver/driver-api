use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct DriverMailTypeRow {
    pub pk_driver_mail_type_id: i32,
    pub label: String,
    pub index: i32,
    pub is_editable: bool,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
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

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq, Clone, sqlx::Type)]
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

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct DriverMailAttachmentRow {
    pub pk_driver_mail_attachment_id: Uuid,
    pub fk_driver_mail_id: Uuid,
    pub file_name: String,
    pub s3_file_path: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct DriverMailAttachment {
    pub pk_driver_mail_attachment_id: Uuid,
    pub file_name: String,
    pub created_at: DateTime<Utc>,
}

impl DriverMailAttachmentRow {
    pub fn to_driver_mail_attachment(&self) -> DriverMailAttachment {
        DriverMailAttachment {
            pk_driver_mail_attachment_id: self.pk_driver_mail_attachment_id,
            file_name: self.file_name.clone(),
            created_at: self.created_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct DriverMail {
    pub pk_driver_mail_id: Uuid,
    pub mail_type: DriverMailType,
    pub email_used: String,
    pub status: MailStatus,
    pub description: String,
    pub content: Option<String>,
    pub created_at: DateTime<Utc>,
    pub sent_at: Option<DateTime<Utc>>,
    pub attachments: Vec<DriverMailAttachment>,
}

impl DriverMailRow {
    pub fn to_driver_mail(
        &self,
        mail_type: DriverMailType,
        attachments: Vec<DriverMailAttachment>,
    ) -> DriverMail {
        DriverMail {
            pk_driver_mail_id: self.pk_driver_mail_id,
            mail_type,
            email_used: self.email_used.clone(),
            status: self.status.clone(),
            description: self.description.clone(),
            content: self.content.clone(),
            created_at: self.created_at,
            sent_at: self.sent_at,
            attachments,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct DriverMailPreference {
    pub mail_type_id: i32,
    pub label: String,
    pub is_editable: bool,
    pub is_enabled: bool,
}

impl DriverMailTypeRow {
    pub fn to_mail_preference(&self, mail_preferences_bitmask: i32) -> DriverMailPreference {
        let bit = 1 << (self.pk_driver_mail_type_id - 1);
        DriverMailPreference {
            mail_type_id: self.pk_driver_mail_type_id,
            label: self.label.clone(),
            is_editable: self.is_editable,
            is_enabled: (mail_preferences_bitmask & bit) != 0,
        }
    }
}

#[derive(Deserialize, Validate, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct GetMailsParams {
    #[validate(range(min = 1, message = "page must be at least 1"))]
    pub page: u32,

    #[validate(range(min = 1, max = 100, message = "limit must be between 1 and 100"))]
    pub limit: u32,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateMailPreferenceRequest {
    pub is_enabled: bool,
}
