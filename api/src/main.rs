use api::{ApiError, app::App};
use dotenv::dotenv;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::trace::TracerProvider;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use api::config::Config;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<(), ApiError> {
    dotenv().ok();

    let config: Config = Config::parse();

    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(&config.otel.exporter_otlp_endpoint)
        .build()
        .expect("failed to build OTLP exporter");

    let provider = TracerProvider::builder()
        .with_batch_exporter(exporter, opentelemetry_sdk::runtime::Tokio)
        .build();

    opentelemetry::global::set_tracer_provider(provider.clone());
    let tracer = provider.tracer(config.otel.service_name.clone());

    let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer()) // console logs
        .with(otel_layer) // traces → otel collector
        .init();

    tracing::info!(
        service_name = %config.otel.service_name,
        otlp_endpoint = %config.otel.exporter_otlp_endpoint,
        "OpenTelemetry initialized"
    );

    let app = App::new(config).await?;
    app.start().await?;

    // flush traces before exit
    provider.shutdown().expect("failed to shutdown OpenTelemetry");

    Ok(())
}
