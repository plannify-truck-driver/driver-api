use chrono::{NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;
use validator::Validate;

use crate::domain::common::entities::{validate_date, validate_time};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Workday {
    pub date: NaiveDate,
    pub start_time: NaiveTime,
    pub end_time: Option<NaiveTime>,
    pub rest_time: NaiveTime,
    pub overnight_rest: bool,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
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

#[derive(Debug, Deserialize, Validate, ToSchema)]
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
    pub date: Option<NaiveDate>,

    #[validate(custom(
        function = "validate_time",
        message = "start_time must be a valid time"
    ))]
    pub start_time: Option<NaiveTime>,

    #[validate(custom(function = "validate_time", message = "end_time must be a valid time"))]
    pub end_time: Option<NaiveTime>,

    #[validate(custom(function = "validate_time", message = "rest_time must be a valid time"))]
    pub rest_time: Option<NaiveTime>,

    pub overnight_rest: Option<bool>,
}
