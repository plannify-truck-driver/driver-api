use api::{ApiError, app::App};
use tracing_subscriber::EnvFilter;
use dotenv::dotenv;

use api::config::Config;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<(), ApiError> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // Load environment variables from .env file
    dotenv().ok();

    let config: Config = Config::parse();
    let app = App::new(config).await?;
    app.start().await?;
    Ok(())
}
