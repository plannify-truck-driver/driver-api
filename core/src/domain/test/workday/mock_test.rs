#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::domain::{
        test::create_mock_service,
        workday::{
            entities::{CreateWorkdayRequest, UpdateWorkdayRequest},
            port::{WorkdayRepository, WorkdayService},
        },
    };

    #[tokio::test]
    async fn test_get_workdays_by_month_success() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();

        // Add dataset
        service
            .workday_repository
            .create_workday(
                Uuid::parse_str("123e4567-e89b-12d3-a456-426614174001").unwrap(),
                CreateWorkdayRequest {
                    date: chrono::NaiveDate::parse_from_str("2026-01-01", "%Y-%m-%d").unwrap(),
                    start_time: chrono::NaiveTime::parse_from_str("08:00:00", "%H:%M:%S").unwrap(),
                    end_time: Some(
                        chrono::NaiveTime::parse_from_str("17:00:00", "%H:%M:%S").unwrap(),
                    ),
                    rest_time: chrono::NaiveTime::parse_from_str("01:00:00", "%H:%M:%S").unwrap(),
                    overnight_rest: false,
                },
            )
            .await?;
        service
            .workday_repository
            .create_workday(
                Uuid::parse_str("123e4567-e89b-12d3-a456-426614174001").unwrap(),
                CreateWorkdayRequest {
                    date: chrono::NaiveDate::parse_from_str("2026-01-02", "%Y-%m-%d").unwrap(),
                    start_time: chrono::NaiveTime::parse_from_str("08:00:00", "%H:%M:%S").unwrap(),
                    end_time: Some(
                        chrono::NaiveTime::parse_from_str("17:00:00", "%H:%M:%S").unwrap(),
                    ),
                    rest_time: chrono::NaiveTime::parse_from_str("01:00:00", "%H:%M:%S").unwrap(),
                    overnight_rest: false,
                },
            )
            .await?;
        service
            .workday_repository
            .create_workday(
                Uuid::parse_str("123e4567-e89b-12d3-a456-426614174001").unwrap(),
                CreateWorkdayRequest {
                    date: chrono::NaiveDate::parse_from_str("2027-01-01", "%Y-%m-%d").unwrap(),
                    start_time: chrono::NaiveTime::parse_from_str("08:00:00", "%H:%M:%S").unwrap(),
                    end_time: Some(
                        chrono::NaiveTime::parse_from_str("17:00:00", "%H:%M:%S").unwrap(),
                    ),
                    rest_time: chrono::NaiveTime::parse_from_str("01:00:00", "%H:%M:%S").unwrap(),
                    overnight_rest: false,
                },
            )
            .await?;
        service
            .workday_repository
            .create_workday(
                Uuid::parse_str("123e4567-e89b-12d3-a456-426614174002").unwrap(),
                CreateWorkdayRequest {
                    date: chrono::NaiveDate::parse_from_str("2026-01-01", "%Y-%m-%d").unwrap(),
                    start_time: chrono::NaiveTime::parse_from_str("08:00:00", "%H:%M:%S").unwrap(),
                    end_time: Some(
                        chrono::NaiveTime::parse_from_str("17:00:00", "%H:%M:%S").unwrap(),
                    ),
                    rest_time: chrono::NaiveTime::parse_from_str("01:00:00", "%H:%M:%S").unwrap(),
                    overnight_rest: false,
                },
            )
            .await?;

        // Test the get_workdays_by_month method
        let workdays = service
            .get_workdays_by_month(
                Uuid::parse_str("123e4567-e89b-12d3-a456-426614174001").unwrap(),
                1,
                2026,
            )
            .await
            .expect("get_workdays_by_month returned an error");

        assert_eq!(workdays.len(), 2, "Expected two workdays in the list");

        Ok(())
    }

    #[tokio::test]
    async fn test_get_workdays_by_period_success() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();

        // Add dataset
        service
            .workday_repository
            .create_workday(
                Uuid::parse_str("123e4567-e89b-12d3-a456-426614174001").unwrap(),
                CreateWorkdayRequest {
                    date: chrono::NaiveDate::parse_from_str("2026-01-01", "%Y-%m-%d").unwrap(),
                    start_time: chrono::NaiveTime::parse_from_str("08:00:00", "%H:%M:%S").unwrap(),
                    end_time: Some(
                        chrono::NaiveTime::parse_from_str("17:00:00", "%H:%M:%S").unwrap(),
                    ),
                    rest_time: chrono::NaiveTime::parse_from_str("01:00:00", "%H:%M:%S").unwrap(),
                    overnight_rest: false,
                },
            )
            .await?;
        service
            .workday_repository
            .create_workday(
                Uuid::parse_str("123e4567-e89b-12d3-a456-426614174001").unwrap(),
                CreateWorkdayRequest {
                    date: chrono::NaiveDate::parse_from_str("2026-01-02", "%Y-%m-%d").unwrap(),
                    start_time: chrono::NaiveTime::parse_from_str("08:00:00", "%H:%M:%S").unwrap(),
                    end_time: Some(
                        chrono::NaiveTime::parse_from_str("17:00:00", "%H:%M:%S").unwrap(),
                    ),
                    rest_time: chrono::NaiveTime::parse_from_str("01:00:00", "%H:%M:%S").unwrap(),
                    overnight_rest: false,
                },
            )
            .await?;
        service
            .workday_repository
            .create_workday(
                Uuid::parse_str("123e4567-e89b-12d3-a456-426614174001").unwrap(),
                CreateWorkdayRequest {
                    date: chrono::NaiveDate::parse_from_str("2026-01-31", "%Y-%m-%d").unwrap(),
                    start_time: chrono::NaiveTime::parse_from_str("08:00:00", "%H:%M:%S").unwrap(),
                    end_time: Some(
                        chrono::NaiveTime::parse_from_str("17:00:00", "%H:%M:%S").unwrap(),
                    ),
                    rest_time: chrono::NaiveTime::parse_from_str("01:00:00", "%H:%M:%S").unwrap(),
                    overnight_rest: false,
                },
            )
            .await?;
        service
            .workday_repository
            .create_workday(
                Uuid::parse_str("123e4567-e89b-12d3-a456-426614174001").unwrap(),
                CreateWorkdayRequest {
                    date: chrono::NaiveDate::parse_from_str("2027-01-01", "%Y-%m-%d").unwrap(),
                    start_time: chrono::NaiveTime::parse_from_str("08:00:00", "%H:%M:%S").unwrap(),
                    end_time: Some(
                        chrono::NaiveTime::parse_from_str("17:00:00", "%H:%M:%S").unwrap(),
                    ),
                    rest_time: chrono::NaiveTime::parse_from_str("01:00:00", "%H:%M:%S").unwrap(),
                    overnight_rest: false,
                },
            )
            .await?;
        service
            .workday_repository
            .create_workday(
                Uuid::parse_str("123e4567-e89b-12d3-a456-426614174002").unwrap(),
                CreateWorkdayRequest {
                    date: chrono::NaiveDate::parse_from_str("2026-01-01", "%Y-%m-%d").unwrap(),
                    start_time: chrono::NaiveTime::parse_from_str("08:00:00", "%H:%M:%S").unwrap(),
                    end_time: Some(
                        chrono::NaiveTime::parse_from_str("17:00:00", "%H:%M:%S").unwrap(),
                    ),
                    rest_time: chrono::NaiveTime::parse_from_str("01:00:00", "%H:%M:%S").unwrap(),
                    overnight_rest: false,
                },
            )
            .await?;

        // Test the get_workdays_by_period method
        let workdays = service
            .get_workdays_by_period(
                Uuid::parse_str("123e4567-e89b-12d3-a456-426614174001").unwrap(),
                chrono::NaiveDate::parse_from_str("2026-01-01", "%Y-%m-%d").unwrap(),
                chrono::NaiveDate::parse_from_str("2026-01-31", "%Y-%m-%d").unwrap(),
                1,
                2,
            )
            .await
            .expect("get_workdays_by_period returned an error");

        assert_eq!(workdays.0.len(), 2, "Expected two workdays in the list");
        assert_eq!(workdays.1, 3, "Expected total count to be 3");

        Ok(())
    }

    #[tokio::test]
    async fn test_create_workday_success() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();

        // Test the create_workday method
        service
            .create_workday(
                Uuid::parse_str("123e4567-e89b-12d3-a456-426614174001").unwrap(),
                CreateWorkdayRequest {
                    date: chrono::NaiveDate::parse_from_str("2026-01-01", "%Y-%m-%d").unwrap(),
                    start_time: chrono::NaiveTime::parse_from_str("08:00:00", "%H:%M:%S").unwrap(),
                    end_time: Some(
                        chrono::NaiveTime::parse_from_str("17:00:00", "%H:%M:%S").unwrap(),
                    ),
                    rest_time: chrono::NaiveTime::parse_from_str("01:00:00", "%H:%M:%S").unwrap(),
                    overnight_rest: false,
                },
            )
            .await
            .expect("create_workday returned an error");

        let workdays = service
            .workday_repository
            .get_workdays_by_period(
                Uuid::parse_str("123e4567-e89b-12d3-a456-426614174001").unwrap(),
                chrono::NaiveDate::parse_from_str("2026-01-01", "%Y-%m-%d").unwrap(),
                chrono::NaiveDate::parse_from_str("2026-01-31", "%Y-%m-%d").unwrap(),
                1,
                10,
            )
            .await
            .expect("get_workdays_by_period returned an error");

        assert_eq!(workdays.0.len(), 1, "Expected one workday in the list");
        assert_eq!(workdays.1, 1, "Expected total count to be 1");
        Ok(())
    }

    #[tokio::test]
    async fn test_update_workday_success() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();

        // Add dataset
        service
            .workday_repository
            .create_workday(
                Uuid::parse_str("123e4567-e89b-12d3-a456-426614174001").unwrap(),
                CreateWorkdayRequest {
                    date: chrono::NaiveDate::parse_from_str("2026-01-01", "%Y-%m-%d").unwrap(),
                    start_time: chrono::NaiveTime::parse_from_str("08:00:00", "%H:%M:%S").unwrap(),
                    end_time: Some(
                        chrono::NaiveTime::parse_from_str("17:00:00", "%H:%M:%S").unwrap(),
                    ),
                    rest_time: chrono::NaiveTime::parse_from_str("01:00:00", "%H:%M:%S").unwrap(),
                    overnight_rest: false,
                },
            )
            .await?;

        // Test the update_workday method
        service
            .update_workday(
                Uuid::parse_str("123e4567-e89b-12d3-a456-426614174001").unwrap(),
                UpdateWorkdayRequest {
                    date: chrono::NaiveDate::parse_from_str("2026-01-01", "%Y-%m-%d").unwrap(),
                    start_time: chrono::NaiveTime::parse_from_str("09:00:00", "%H:%M:%S").unwrap(),
                    end_time: Some(
                        chrono::NaiveTime::parse_from_str("16:00:00", "%H:%M:%S").unwrap(),
                    ),
                    rest_time: chrono::NaiveTime::parse_from_str("00:45:00", "%H:%M:%S").unwrap(),
                    overnight_rest: true,
                },
            )
            .await
            .expect("create_workday returned an error");

        let workdays = service
            .workday_repository
            .get_workdays_by_period(
                Uuid::parse_str("123e4567-e89b-12d3-a456-426614174001").unwrap(),
                chrono::NaiveDate::parse_from_str("2026-01-01", "%Y-%m-%d").unwrap(),
                chrono::NaiveDate::parse_from_str("2026-01-31", "%Y-%m-%d").unwrap(),
                1,
                10,
            )
            .await
            .expect("get_workdays_by_period returned an error");

        assert_eq!(workdays.0.len(), 1, "Expected one workday in the list");
        assert_eq!(workdays.1, 1, "Expected total count to be 1");
        assert_eq!(
            workdays.0[0].start_time,
            chrono::NaiveTime::parse_from_str("09:00:00", "%H:%M:%S").unwrap(),
            "Expected start_time to be updated"
        );
        assert_eq!(
            workdays.0[0].end_time,
            Some(chrono::NaiveTime::parse_from_str("16:00:00", "%H:%M:%S").unwrap()),
            "Expected end_time to be updated"
        );
        assert_eq!(
            workdays.0[0].rest_time,
            chrono::NaiveTime::parse_from_str("00:45:00", "%H:%M:%S").unwrap(),
            "Expected rest_time to be updated"
        );
        assert_eq!(
            workdays.0[0].overnight_rest, true,
            "Expected overnight_rest to be updated"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_workday_success() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();

        // Add dataset
        service
            .workday_repository
            .create_workday(
                Uuid::parse_str("123e4567-e89b-12d3-a456-426614174001").unwrap(),
                CreateWorkdayRequest {
                    date: chrono::NaiveDate::parse_from_str("2026-01-01", "%Y-%m-%d").unwrap(),
                    start_time: chrono::NaiveTime::parse_from_str("08:00:00", "%H:%M:%S").unwrap(),
                    end_time: Some(
                        chrono::NaiveTime::parse_from_str("17:00:00", "%H:%M:%S").unwrap(),
                    ),
                    rest_time: chrono::NaiveTime::parse_from_str("01:00:00", "%H:%M:%S").unwrap(),
                    overnight_rest: false,
                },
            )
            .await?;

        // Test the delete_workday method
        service
            .delete_workday(
                Uuid::parse_str("123e4567-e89b-12d3-a456-426614174001").unwrap(),
                chrono::NaiveDate::parse_from_str("2026-01-01", "%Y-%m-%d").unwrap(),
            )
            .await
            .expect("delete_workday returned an error");

        let workdays = service
            .workday_repository
            .get_workdays_by_period(
                Uuid::parse_str("123e4567-e89b-12d3-a456-426614174001").unwrap(),
                chrono::NaiveDate::parse_from_str("2026-01-01", "%Y-%m-%d").unwrap(),
                chrono::NaiveDate::parse_from_str("2026-01-31", "%Y-%m-%d").unwrap(),
                1,
                10,
            )
            .await
            .expect("get_workdays_by_period returned an error");

        assert_eq!(workdays.0.len(), 0, "Expected zero workdays in the list");
        assert_eq!(workdays.1, 0, "Expected total count to be 0");

        Ok(())
    }
}
