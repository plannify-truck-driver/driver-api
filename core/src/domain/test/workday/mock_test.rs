#[cfg(test)]
mod tests {
    use chrono::Datelike;
    use uuid::Uuid;

    use crate::{
        domain::{
            test::create_mock_service,
            workday::{
                entities::{CreateWorkdayRequest, UpdateWorkdayRequest},
                port::{WorkdayDatabaseRepository, WorkdayService},
            },
        },
        infrastructure::workday::repositories::error::WorkdayError,
    };

    #[tokio::test]
    async fn test_get_workdays_by_month_success() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();

        // Add dataset
        service
            .workday_database_repository
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
            .workday_database_repository
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
            .workday_database_repository
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
            .workday_database_repository
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

        assert_eq!(
            workdays[0].date.month(),
            1,
            "Expected workday month to be January"
        );
        assert_eq!(
            workdays[0].date.year(),
            2026,
            "Expected workday year to be 2026"
        );

        assert_eq!(
            workdays[1].date.month(),
            1,
            "Expected workday month to be January"
        );
        assert_eq!(
            workdays[1].date.year(),
            2026,
            "Expected workday year to be 2026"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_get_workdays_by_period_success() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();

        // Add dataset
        service
            .workday_database_repository
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
            .workday_database_repository
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
            .workday_database_repository
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
            .workday_database_repository
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
            .workday_database_repository
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

        // Add dataset
        service
            .workday_database_repository
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
            .workday_database_repository
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
    async fn test_create_workday_fail_duplicate() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();

        // Add dataset
        service
            .workday_database_repository
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

        // Test the create_workday method
        let error = service
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
            .expect_err("create_workday should have returned an error");

        assert_eq!(
            error,
            WorkdayError::WorkdayAlreadyExists,
            "Expected duplicate workday request error"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_update_workday_success() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();

        // Add dataset
        service
            .workday_database_repository
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
            .workday_database_repository
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
        assert!(
            workdays.0[0].overnight_rest,
            "Expected overnight_rest to be updated"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_update_workday_fail_not_found() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();

        // Add dataset
        service
            .workday_database_repository
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

        // Test the update_workday method
        let error = service
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
            .expect_err("update_workday should have returned an error");

        assert_eq!(
            error,
            WorkdayError::WorkdayNotFound,
            "Expected workday not found error"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_workday_success() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();

        // Add dataset
        service
            .workday_database_repository
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
            .workday_database_repository
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

    #[tokio::test]
    async fn test_delete_workday_fail_not_found() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();

        // Add dataset
        service
            .workday_database_repository
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

        // Test the delete_workday method
        let result = service
            .delete_workday(
                Uuid::parse_str("123e4567-e89b-12d3-a456-426614174001").unwrap(),
                chrono::NaiveDate::parse_from_str("2026-01-01", "%Y-%m-%d").unwrap(),
            )
            .await
            .expect_err("delete_workday should have returned an error");

        assert_eq!(
            result,
            WorkdayError::WorkdayNotFound,
            "delete_workday should succeed even if workday does not exist"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_get_workdays_garbage_success() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();

        // Add dataset
        service
            .workday_database_repository
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
            .workday_database_repository
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
            .workday_database_repository
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
        service
            .workday_database_repository
            .create_workday_garbage(
                Uuid::parse_str("123e4567-e89b-12d3-a456-426614174001").unwrap(),
                chrono::NaiveDate::parse_from_str("2026-01-01", "%Y-%m-%d").unwrap(),
                chrono::NaiveDate::parse_from_str("2026-02-01", "%Y-%m-%d").unwrap(),
                None,
            )
            .await?;
        service
            .workday_database_repository
            .create_workday_garbage(
                Uuid::parse_str("123e4567-e89b-12d3-a456-426614174002").unwrap(),
                chrono::NaiveDate::parse_from_str("2026-01-01", "%Y-%m-%d").unwrap(),
                chrono::NaiveDate::parse_from_str("2026-02-01", "%Y-%m-%d").unwrap(),
                None,
            )
            .await?;

        // Test the get_workdays_garbage method
        let workdays = service
            .get_workdays_garbage(Uuid::parse_str("123e4567-e89b-12d3-a456-426614174001").unwrap())
            .await
            .expect("get_workdays_garbage returned an error");

        assert_eq!(workdays.len(), 1, "Expected one workday in the list");

        assert_eq!(
            workdays[0].workday_date.day(),
            1,
            "Expected workday day to be 1"
        );
        assert_eq!(
            workdays[0].workday_date.month(),
            1,
            "Expected workday month to be January"
        );
        assert_eq!(
            workdays[0].workday_date.year(),
            2026,
            "Expected workday year to be 2026"
        );
        assert_eq!(
            workdays[0].fk_driver_id,
            Uuid::parse_str("123e4567-e89b-12d3-a456-426614174001").unwrap(),
            "Expected workday driver ID to match"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_get_workdays_garbage_empty() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();

        // Test the get_workdays_garbage method
        let workdays = service
            .get_workdays_garbage(Uuid::parse_str("123e4567-e89b-12d3-a456-426614174001").unwrap())
            .await
            .expect("get_workdays_garbage returned an error");

        assert_eq!(workdays.len(), 0, "Expected zero workdays in the list");

        Ok(())
    }

    #[tokio::test]
    async fn test_create_workday_garbage_success() -> Result<(), Box<dyn std::error::Error>> {
        let service = create_mock_service();

        // Add dataset
        service
            .workday_database_repository
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

        let workday = service
            .create_workday_garbage(
                Uuid::parse_str("123e4567-e89b-12d3-a456-426614174001").unwrap(),
                chrono::NaiveDate::parse_from_str("2026-01-01", "%Y-%m-%d").unwrap(),
            )
            .await
            .expect("create_workday_garbage returned an error");

        assert_eq!(
            workday.fk_driver_id,
            Uuid::parse_str("123e4567-e89b-12d3-a456-426614174001").unwrap(),
            "Expected workday driver ID to match"
        );
        assert_eq!(
            workday.workday_date,
            chrono::NaiveDate::parse_from_str("2026-01-01", "%Y-%m-%d").unwrap(),
            "Expected workday date to match"
        );

        Ok(())
    }
}
