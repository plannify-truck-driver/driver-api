use bytes::Bytes;
use tonic::transport::Channel;
use tracing::instrument;

use crate::domain::{document::port::DocumentExternalRepository, workday::entities::Workday};
use crate::infrastructure::document::repositories::{
    error::DocumentError,
    proto::{
        GenerateMonthlyWorkdayReportRequest, Language as ProtoLanguage, Workday as ProtoWorkday,
        workday_service_client::WorkdayServiceClient,
    },
};

use tracing::error;

#[derive(Clone)]
pub struct GrpcDocumentRepository {
    channel: Channel,
}

impl GrpcDocumentRepository {
    pub fn new(channel: Channel) -> Self {
        Self { channel }
    }

    pub async fn connect(
        endpoint: impl Into<String>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let endpoint = endpoint.into();
        let channel = Channel::from_shared(endpoint.clone())
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })?
            .connect()
            .await
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })?;
        Ok(Self::new(channel))
    }
}

fn workday_to_proto(w: &Workday) -> ProtoWorkday {
    ProtoWorkday {
        date: w.date.format("%Y-%m-%d").to_string(),
        start_time: w.start_time.format("%H:%M:%S").to_string(),
        end_time: w.end_time.map(|t| t.format("%H:%M:%S").to_string()),
        rest_time: w.rest_time.format("%H:%M:%S").to_string(),
        overnight: w.overnight_rest,
    }
}

fn language_to_proto(language: &str) -> ProtoLanguage {
    let s = language.to_lowercase();
    if s == "fr" || s == "french" || s == "français" {
        ProtoLanguage::French
    } else {
        ProtoLanguage::English
    }
}

impl DocumentExternalRepository for GrpcDocumentRepository {
    #[instrument(skip(self, workdays))]
    async fn get_workday_documents_by_month(
        &self,
        driver_firstname: String,
        driver_lastname: String,
        language: String,
        month: i32,
        year: i32,
        workdays: Vec<Workday>,
    ) -> Result<Option<Bytes>, DocumentError> {
        let month_u32 = u32::try_from(month).map_err(|_| DocumentError::Internal)?;
        let year_u32 = u32::try_from(year).map_err(|_| DocumentError::Internal)?;

        let request = GenerateMonthlyWorkdayReportRequest {
            driver_firstname,
            driver_lastname,
            language: language_to_proto(&language) as i32,
            month: month_u32,
            year: year_u32,
            workdays: workdays.iter().map(workday_to_proto).collect(),
        };

        let mut client = WorkdayServiceClient::new(self.channel.clone());
        let response = client
            .generate_monthly_workday_report(request)
            .await
            .map_err(|e| {
                error!(error = %e, "gRPC call to WorkdayService failed");
                DocumentError::Internal
            })?;

        let pdf_content = response.into_inner().pdf_content;
        Ok(if pdf_content.is_empty() {
            None
        } else {
            Some(Bytes::from(pdf_content))
        })
    }
}
