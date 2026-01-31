use std::future::Future;

use bytes::Bytes;

use crate::{
    domain::workday::entities::Workday,
    infrastructure::document::repositories::error::DocumentError,
};

pub trait DocumentExternalRepository: Send + Sync {
    fn get_workday_documents_by_month(
        &self,
        driver_firstname: String,
        driver_lastname: String,
        language: String,
        month: i32,
        year: i32,
        workdays: Vec<Workday>,
    ) -> impl Future<Output = Result<Option<Bytes>, DocumentError>> + Send;
}

#[derive(Clone, Default)]
pub struct MockDocumentExternalRepository;

impl DocumentExternalRepository for MockDocumentExternalRepository {
    async fn get_workday_documents_by_month(
        &self,
        _driver_firstname: String,
        _driver_lastname: String,
        _language: String,
        _month: i32,
        _year: i32,
        _workdays: Vec<Workday>,
    ) -> Result<Option<bytes::Bytes>, DocumentError> {
        Ok(None)
    }
}
