use clap::{Parser, Subcommand};
use dotenv::dotenv;
use plannify_driver_api_core::{ServiceConfig, application::create_repositories};
use tracing_subscriber::EnvFilter;

mod config;
mod jobs;

use config::Config;

#[derive(Parser)]
#[command(name = "job", about = "Plannify Driver job runner")]
struct Cli {
    #[command(subcommand)]
    command: JobCommand,

    #[command(flatten)]
    config: Config,
}

#[derive(Subcommand)]
enum JobCommand {
    /// Delete expired workday garbage entries
    DeleteGarbage,

    /// Generate workday documents for months older than N months that have no document yet
    GenerateDocuments {
        #[arg(long, help = "Number of months in the past to use as cutoff")]
        months_ago: u32,
    },
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let cli = Cli::parse();
    let config = &cli.config;

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let repos = match create_repositories(
        &config.database_url,
        &config.redis_url,
        config.smtp.to_client(),
        config.smtp.to_transport(),
        config.frontend_url.clone(),
        false,
        &config.pdf_service_endpoint,
        &config.s3.access_key,
        &config.s3.secret_key,
        &config.s3.endpoint,
        &config.s3.region,
        &config.s3.bucket_name,
        ServiceConfig {
            workday_garbage_retention_days: config.workday_garbage_retention_days,
            account_deactivation_days: 30,
        },
    )
    .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Failed to initialize repositories: {}", e);
            std::process::exit(1);
        }
    };

    let exit_code = match cli.command {
        JobCommand::DeleteGarbage => jobs::delete_garbage::run(&repos).await,
        JobCommand::GenerateDocuments { months_ago } => {
            jobs::generate_documents::run(&repos, months_ago).await
        }
    };

    repos.shutdown_pool().await;

    std::process::exit(exit_code);
}
