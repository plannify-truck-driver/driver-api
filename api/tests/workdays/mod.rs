use plannify_driver_api_core::domain::workday::entities::Workday;

pub mod get_all_workdays_month;
pub mod get_all_workdays_period;
pub mod create_workday;
pub mod update_workday;
pub mod delete_workday;
pub mod get_all_workday_garbage;
pub mod delete_workday_garbage;
pub mod get_workday_documents;

fn verify_workday_content(workday: Workday, expected_workday: Workday) {
    assert_eq!(workday.date, expected_workday.date);
    assert_eq!(workday.start_time, expected_workday.start_time);
    assert_eq!(workday.end_time, expected_workday.end_time);
    assert_eq!(workday.rest_time, expected_workday.rest_time);
    assert_eq!(workday.overnight_rest, expected_workday.overnight_rest);
}