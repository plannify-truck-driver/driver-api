use plannify_driver_api_core::domain::workday::entities::{Workday, WorkdayDocumentInformation};

pub mod create_workday;
pub mod delete_workday;
pub mod delete_workday_garbage;
pub mod get_all_workday_garbage;
pub mod get_all_workdays_month;
pub mod get_all_workdays_period;
pub mod get_workday_by_date;
pub mod get_workday_document_by_month;
pub mod get_workday_documents;
pub mod update_workday;

fn verify_workday_content(workday: Workday, expected_workday: Workday) {
    assert_eq!(workday.date, expected_workday.date);
    assert_eq!(workday.start_time, expected_workday.start_time);
    assert_eq!(workday.end_time, expected_workday.end_time);
    assert_eq!(workday.rest_time, expected_workday.rest_time);
    assert_eq!(workday.overnight_rest, expected_workday.overnight_rest);
}

fn verify_workday_document_content(
    document: WorkdayDocumentInformation,
    expected_document: WorkdayDocumentInformation,
) {
    assert_eq!(document.month, expected_document.month);
    assert_eq!(document.year, expected_document.year);
    assert_eq!(document.generated_at, expected_document.generated_at);
}
