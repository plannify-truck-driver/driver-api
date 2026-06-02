use chrono::{Datelike, NaiveDate, NaiveTime, Timelike};
use validator::ValidationError;

use crate::domain::driver::entities::DriverLanguage;

pub fn validate_language(lang: &DriverLanguage) -> Result<(), ValidationError> {
    match lang {
        DriverLanguage::FR | DriverLanguage::EN => Ok(()),
    }
}

pub fn validate_phone_number(phone: &str) -> Result<(), ValidationError> {
    if !phone.starts_with('+') {
        return Err(ValidationError::new("phone_format"));
    }
    let digits = &phone[1..];
    if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit()) {
        return Err(ValidationError::new("phone_format"));
    }
    let len = digits.len();
    if !(7..=15).contains(&len) {
        return Err(ValidationError::new("phone_format"));
    }
    if digits.starts_with('0') {
        return Err(ValidationError::new("phone_format"));
    }
    Ok(())
}

pub fn validate_date(date: &NaiveDate) -> Result<(), ValidationError> {
    if !(1900..=2100).contains(&date.year()) {
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
