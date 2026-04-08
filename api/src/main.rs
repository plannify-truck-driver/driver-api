use api::{ApiError, app::App};
use dotenv::dotenv;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_sdk::trace::TracerProvider;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use api::config::Config;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<(), ApiError> {
    dotenv().ok();

    // 1. Initialiser l'exporter OTLP (gRPC/tonic)
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .build()
        .expect("Erreur init OTLP exporter");

    // 2. Créer le provider avec batch processing
    let provider = TracerProvider::builder()
        .with_batch_exporter(exporter, opentelemetry_sdk::runtime::Tokio)
        .build();

    opentelemetry::global::set_tracer_provider(provider.clone());
    let tracer = provider.tracer("api");

    // 3. Créer le layer OTEL pour tracing
    let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    // 4. Initialiser le subscriber avec les deux layers
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer())  // logs console
        .with(otel_layer)                         // traces → otel-collector
        .init();

    tracing::info!("OpenTelemetry initialisé");

    let config: Config = Config::parse();
    let app = App::new(config).await?;
    app.start().await?;

    // 5. Flush des traces avant de quitter
    provider.shutdown().expect("Erreur shutdown OpenTelemetry");

    Ok(())
}