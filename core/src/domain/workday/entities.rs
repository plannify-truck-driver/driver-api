use chrono::{DateTime, Datelike, Months, NaiveDate, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;
use validator::Validate;

use crate::domain::common::entities::{validate_date, validate_time};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, ToSchema)]
pub struct Workday {
    pub date: NaiveDate,
    pub start_time: NaiveTime,
    pub end_time: Option<NaiveTime>,
    pub rest_time: NaiveTime,
    pub overnight_rest: bool,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct WorkdayRow {
    pub date: NaiveDate,
    pub start_time: NaiveTime,
    pub end_time: Option<NaiveTime>,
    pub rest_time: NaiveTime,
    pub overnight_rest: bool,
    pub fk_driver_id: Uuid,
}

impl WorkdayRow {
    pub fn to_workday(&self) -> Workday {
        Workday {
            date: self.date,
            start_time: self.start_time,
            end_time: self.end_time,
            rest_time: self.rest_time,
            overnight_rest: self.overnight_rest,
        }
    }
}

#[derive(Deserialize, Validate, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct GetWorkdaysByMonthParams {
    #[validate(range(min = 1, max = 12, message = "month must be between 1 and 12"))]
    pub month: i32,

    #[validate(range(min = 1900, max = 2100, message = "year must be between 1900 and 2100"))]
    pub year: i32,
}

#[derive(Deserialize, Validate, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct GetWorkdaysByPeriodParams {
    #[validate(custom(
        function = "validate_date",
        message = "date must be between 1900 and 2100"
    ))]
    pub from: NaiveDate,

    #[validate(custom(
        function = "validate_date",
        message = "date must be between 1900 and 2100"
    ))]
    pub to: NaiveDate,

    #[validate(range(min = 1, message = "page must be at least 1"))]
    pub page: u32,

    #[validate(range(min = 1, max = 100, message = "limit must be between 1 and 100"))]
    pub limit: u32,
}

#[derive(Debug, Deserialize, Clone, Copy, Validate, ToSchema)]
pub struct CreateWorkdayRequest {
    #[validate(custom(
        function = "validate_date",
        message = "date must be between 1900 and 2100"
    ))]
    pub date: NaiveDate,

    #[validate(custom(
        function = "validate_time",
        message = "start_time must be a valid time"
    ))]
    pub start_time: NaiveTime,

    #[validate(custom(function = "validate_time", message = "end_time must be a valid time"))]
    pub end_time: Option<NaiveTime>,

    #[validate(custom(function = "validate_time", message = "rest_time must be a valid time"))]
    pub rest_time: NaiveTime,

    pub overnight_rest: bool,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct UpdateWorkdayRequest {
    #[validate(custom(
        function = "validate_date",
        message = "date must be between 1900 and 2100"
    ))]
    pub date: NaiveDate,

    #[validate(custom(
        function = "validate_time",
        message = "start_time must be a valid time"
    ))]
    pub start_time: NaiveTime,

    #[validate(custom(function = "validate_time", message = "end_time must be a valid time"))]
    pub end_time: Option<NaiveTime>,

    #[validate(custom(function = "validate_time", message = "rest_time must be a valid time"))]
    pub rest_time: NaiveTime,

    pub overnight_rest: bool,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct WorkdayGarbageRow {
    pub workday_date: NaiveDate,
    pub fk_driver_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub scheduled_deletion_date: NaiveDate,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct WorkdayGarbage {
    pub workday_date: NaiveDate,
    pub created_at: DateTime<Utc>,
    pub scheduled_deletion_date: NaiveDate,
}

impl WorkdayGarbageRow {
    pub fn to_workday_garbage(&self) -> WorkdayGarbage {
        WorkdayGarbage {
            workday_date: self.workday_date,
            created_at: self.created_at,
            scheduled_deletion_date: self.scheduled_deletion_date,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, ToSchema)]
pub struct WorkdayDocumentInformation {
    pub month: u32,
    pub year: u32,
    pub generated_at: Option<DateTime<Utc>>,
}

pub type GetWorkdayDocumentsByYearResponse = Vec<WorkdayDocumentInformation>;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, ToSchema)]
pub struct UpcomingWorkdayDocument {
    pub month: i32,
    pub year: i32,
}

/// First day of the month `months_ago` months before `run_date`: the `generate-documents`
/// job generates documents for every month strictly before this date.
pub fn document_generation_cutoff(run_date: NaiveDate, months_ago: u32) -> NaiveDate {
    let cutoff = run_date - Months::new(months_ago);
    NaiveDate::from_ymd_opt(cutoff.year(), cutoff.month(), 1).expect("cutoff date is always valid")
}

/// Accounts younger than this get no document generated.
pub const DOCUMENT_GENERATION_MIN_ACCOUNT_AGE_MONTHS: u32 = 6;

/// Only drivers whose account was created strictly before this date are
/// eligible for document generation on `run_date`.
pub fn document_generation_account_cutoff(run_date: NaiveDate) -> NaiveDate {
    run_date - Months::new(DOCUMENT_GENERATION_MIN_ACCOUNT_AGE_MONTHS)
}

#[cfg(test)]
mod document_generation_cutoff_tests {
    use super::{document_generation_account_cutoff, document_generation_cutoff};
    use chrono::NaiveDate;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    #[test]
    fn test_cutoff() {
        assert_eq!(document_generation_cutoff(d(2026, 11, 1), 3), d(2026, 8, 1));
        assert_eq!(document_generation_cutoff(d(2026, 2, 1), 3), d(2025, 11, 1));
        assert_eq!(document_generation_cutoff(d(2026, 5, 31), 3), d(2026, 2, 1));
        assert_eq!(
            document_generation_cutoff(d(2026, 11, 1), 0),
            d(2026, 11, 1)
        );
    }

    #[test]
    fn test_account_cutoff() {
        assert_eq!(
            document_generation_account_cutoff(d(2026, 11, 1)),
            d(2026, 5, 1)
        );
        assert_eq!(
            document_generation_account_cutoff(d(2026, 2, 1)),
            d(2025, 8, 1)
        );
        // Clamped to the last day of a shorter month
        assert_eq!(
            document_generation_account_cutoff(d(2026, 8, 31)),
            d(2026, 2, 28)
        );
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, ToSchema)]
pub struct WorkdayCreationLimit {
    pub limit: i64,
    pub remaining: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct WorkdayDocumentRow {
    pub fk_driver_id: Uuid,
    pub month: i32,
    pub year: i32,
    pub fk_document_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkdayDocument {
    pub fk_driver_id: Uuid,
    pub month: i32,
    pub year: i32,
    pub s3_file_path: String,
    pub file_name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct DocumentRow {
    pub pk_document_id: Uuid,
    pub s3_file_path: String,
    pub file_name: String,
    pub created_at: DateTime<Utc>,
}
