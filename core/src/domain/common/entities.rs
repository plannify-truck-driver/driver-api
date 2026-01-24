use chrono::{Datelike, NaiveDate, NaiveTime, Timelike};
use validator::ValidationError;

use crate::domain::driver::entities::DriverLanguage;

pub fn validate_language(lang: &DriverLanguage) -> Result<(), ValidationError> {
    match lang {
        DriverLanguage::FR | DriverLanguage::EN => Ok(()),
    }
}

pub fn validate_date(date: &NaiveDate) -> Result<(), ValidationError> {
    if date.year() < 1900 || date.year() > 2100 {
        return Err(ValidationError::new("date must be between 1900 and 2100"));
    }
    Ok(())
}

pub fn validate_time(time: &NaiveTime) -> Result<(), ValidationError> {
    if time.hour() > 23 || time.minute() > 59 || time.second() > 59 {
        return Err(ValidationError::new(
            "time must be between 00:00:00 and 23:59:59",
        ));
    }
    Ok(())
}
