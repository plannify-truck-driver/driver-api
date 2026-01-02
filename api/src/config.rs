use clap::Parser;
use clap::ValueEnum;
use sqlx::postgres::PgConnectOptions;

#[derive(Clone, Parser, Debug, Default)]
#[command(name = "driver-api")]
#[command(about = "Driver API Server", long_about = None)]
pub struct Config {
    #[command(flatten)]
    pub database: DatabaseConfig,

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
pub struct DatabaseConfig {
    #[arg(
        long = "database-host",
        env = "DATABASE_HOST",
        default_value = "localhost"
    )]
    pub host: String,

    #[arg(long = "database-port", env = "DATABASE_PORT", default_value = "5432")]
    pub port: u16,

    #[arg(
        long = "database-user",
        env = "DATABASE_USER",
        default_value = "postgres"
    )]
    pub user: String,

    #[arg(
        long = "database-password",
        env = "DATABASE_PASSWORD",
        value_name = "database_password"
    )]
    pub password: String,

    #[arg(
        long = "database-name",
        env = "DATABASE_NAME",
        default_value = "communities",
        value_name = "database_name"
    )]
    pub db_name: String,
}

impl Into<PgConnectOptions> for DatabaseConfig {
    fn into(self) -> PgConnectOptions {
        PgConnectOptions::new()
            .host(&self.host)
            .port(self.port)
            .username(&self.user)
            .password(&self.password)
            .database(&self.db_name)
    }
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
    pub access_ttl: String,

    #[arg(
        long = "jwt-refresh-ttl",
        env = "JWT_REFRESH_TTL",
        name = "jwt_refresh_ttl"
    )]
    pub refresh_ttl: String,
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

    #[arg(
        long = "cors-origins",
        env = "CORS_ORIGINS",
        value_delimiter = ','
    )]
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
