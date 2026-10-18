use std::{collections::HashSet, net::SocketAddr, time::Duration};

use bytes::Bytes;
use tokio::net::lookup_host;
use tonic::transport::{Channel, Endpoint, Uri, channel::Change};

use crate::domain::{document::port::DocumentExternalRepository, workday::entities::Workday};
use crate::infrastructure::document::repositories::{
    error::DocumentError,
    proto::{
        GenerateMonthlyWorkdayReportRequest, Language as ProtoLanguage, Workday as ProtoWorkday,
        workday_service_client::WorkdayServiceClient,
    },
};

use tracing::{error, warn};

#[derive(Clone)]
pub struct GrpcDocumentRepository {
    channel: Channel,
}

impl GrpcDocumentRepository {
    pub fn new(channel: Channel) -> Self {
        Self { channel }
    }

    /// Resolves every address behind the endpoint host (one per pod with a headless
    /// Kubernetes Service) and balances requests across them. DNS is re-resolved
    /// periodically so scaled-up/down pods are picked up.
    pub async fn connect(
        endpoint: impl Into<String>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let uri: Uri = endpoint.into().parse()?;
        let scheme = uri.scheme_str().unwrap_or("http").to_string();
        let host = uri.host().ok_or("missing host in endpoint")?.to_string();
        let port = uri.port_u16().unwrap_or(80);

        let mut known = resolve(&host, port).await?;
        if known.is_empty() {
            return Err(format!("no address found for {host}:{port}").into());
        }

        let (channel, tx) = Channel::balance_channel::<SocketAddr>(64);
        for addr in &known {
            tx.send(Change::Insert(*addr, endpoint_for(&scheme, *addr)?))
                .await?;
        }

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(DNS_REFRESH_INTERVAL);
            interval.tick().await;
            loop {
                interval.tick().await;
                let fresh = match resolve(&host, port).await {
                    Ok(fresh) if !fresh.is_empty() => fresh,
                    Ok(_) => continue,
                    Err(e) => {
                        warn!(error = %e, host = %host, "Failed to resolve document service");
                        continue;
                    }
                };

                for addr in known.difference(&fresh) {
                    if tx.send(Change::Remove(*addr)).await.is_err() {
                        return;
                    }
                }
                for addr in fresh.difference(&known) {
                    let Ok(endpoint) = endpoint_for(&scheme, *addr) else {
                        continue;
                    };
                    if tx.send(Change::Insert(*addr, endpoint)).await.is_err() {
                        return;
                    }
                }
                known = fresh;
            }
        });

        Ok(Self::new(channel))
    }
}

const DNS_REFRESH_INTERVAL: Duration = Duration::from_secs(30);

async fn resolve(host: &str, port: u16) -> std::io::Result<HashSet<SocketAddr>> {
    let mut addrs: HashSet<SocketAddr> = lookup_host((host, port)).await?.collect();
    // `localhost` resolves to both ::1 and 127.0.0.1 while the service usually only
    // listens on IPv4: prefer IPv4 when available so no endpoint is dead.
    if addrs.iter().any(SocketAddr::is_ipv4) {
        addrs.retain(SocketAddr::is_ipv4);
    }
    Ok(addrs)
}

fn endpoint_for(scheme: &str, addr: SocketAddr) -> Result<Endpoint, tonic::transport::Error> {
    Endpoint::from_shared(format!("{scheme}://{addr}"))
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
    #[tracing::instrument(
        name = "grpc.documents.get_workday_report",
        skip(self),
        fields(
            driver_firstname = %driver_firstname,
            driver_lastname = %driver_lastname,
            language = %language,
            month = %month,
            year = %year,
            workday_count = %workdays.len(),
        )
    )]
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

#[cfg(test)]
mod tests {
    use super::resolve;

    #[tokio::test]
    async fn test_resolve_localhost_prefers_ipv4() {
        let addrs = resolve("localhost", 4000).await.unwrap();
        assert!(!addrs.is_empty());
        assert!(addrs.iter().all(|a| a.is_ipv4() && a.port() == 4000));
    }
}
