use clap::Parser;
use clap::ValueEnum;

#[derive(Clone, Parser, Debug, Default)]
#[command(name = "driver-api")]
#[command(about = "Driver API Server", long_about = None)]
pub struct Config {
    #[arg(
        long = "database-url",
        env = "DATABASE_URL",
        default_value = "postgres://postgres:password@localhost:5432/plannify"
    )]
    pub database_url: String,

    #[command(flatten)]
    pub jwt: JwtConfig,

    #[command(flatten)]
    pub common: CommonConfig,

    #[command(flatten)]
    pub check_content: CheckContentConfig,

    #[arg(
        long = "environment",
        env = "ENVIRONMENT",
        default_value = "development"
    )]
    pub environment: Environment,
}

#[derive(Clone, Parser, Debug, Default)]
pub struct JwtConfig {
    #[arg(
        long = "jwt-secret-key",
        env = "JWT_SECRET_KEY",
        name = "jwt_secret_key"
    )]
    pub secret_key: String,

    #[arg(
        long = "jwt-access-ttl",
        env = "JWT_ACCESS_TTL",
        name = "jwt_access_ttl"
    )]
    pub access_ttl: u64,

    #[arg(
        long = "jwt-refresh-ttl",
        env = "JWT_REFRESH_TTL",
        name = "jwt_refresh_ttl"
    )]
    pub refresh_ttl: u64,
}

#[derive(Clone, Parser, Debug, Default)]
pub struct CommonConfig {
    #[arg(
        long = "server-api-port",
        env = "API_PORT",
        default_value = "8080",
        name = "api_port"
    )]
    pub api_port: u16,

    #[arg(
        long = "server-health-port",
        env = "HEALTH_PORT",
        default_value = "8081"
    )]
    pub health_port: u16,

    #[arg(long = "cors-origins", env = "CORS_ORIGINS", value_delimiter = ',')]
    pub origins: Vec<String>,
}

#[derive(Clone, Parser, Debug, Default)]
pub struct CheckContentConfig {
    #[arg(
        long = "email-domain-denylist",
        env = "EMAIL_DOMAIN_DENYLIST",
        value_delimiter = ','
    )]
    pub email_domain_denylist: Vec<String>,
}

#[derive(Clone, Debug, ValueEnum, Default)]
pub enum Environment {
    #[default]
    Development,
    Production,
    Test,
}
