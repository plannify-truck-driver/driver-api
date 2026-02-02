use clap::Parser;
use clap::ValueEnum;
use lettre::SmtpTransport;
use lettre::message::MessageBuilder;
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;

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

    #[arg(
        long = "redis-url",
        env = "REDIS_URL",
        default_value = "redis://localhost:6379/0"
    )]
    pub redis_url: String,

    #[command(flatten)]
    pub smtp: SmtpConfig,

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
pub struct SmtpConfig {
    #[arg(
        long = "smtp-default-sender",
        env = "SMTP_DEFAULT_SENDER",
        name = "smtp_default_sender"
    )]
    pub default_sender: String,

    #[arg(
        long = "smtp-default-sender-reply-to",
        env = "SMTP_DEFAULT_SENDER_REPLY_TO",
        name = "smtp_default_sender_reply_to"
    )]
    pub default_sender_reply_to: String,

    #[arg(long = "smtp-username", env = "SMTP_USERNAME", name = "smtp_username")]
    pub username: String,

    #[arg(long = "smtp-password", env = "SMTP_PASSWORD", name = "smtp_password")]
    pub password: String,

    #[arg(long = "smtp-domain", env = "SMTP_DOMAIN", name = "smtp_domain")]
    pub domain: String,
}

impl SmtpConfig {
    pub fn to_client(&self) -> MessageBuilder {
        MessageBuilder::new()
            .from(self.default_sender.parse().unwrap())
            .reply_to(self.default_sender_reply_to.parse().unwrap())
            .header(ContentType::TEXT_HTML)
    }

    pub fn to_transport(&self) -> SmtpTransport {
        let creds = Credentials::new(self.username.to_owned(), self.password.to_owned());

        SmtpTransport::relay(&self.domain)
            .unwrap()
            .credentials(creds)
            .build()
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

    #[arg(
        long = "rate-limit-requests-per-second",
        env = "RATE_LIMIT_REQUESTS_PER_SECOND",
        default_value = "100",
        name = "rate_limit_requests_per_second"
    )]
    pub rate_limit_requests: u64,

    #[arg(
        long = "frontend-url",
        env = "FRONTEND_URL",
        default_value = "https://app.plannify.be",
        name = "frontend_url"
    )]
    pub frontend_url: String,

    #[arg(
        long = "pdf-service-endpoint",
        env = "PDF_SERVICE_ENDPOINT",
        default_value = "http://localhost:50051",
        name = "pdf_service_endpoint"
    )]
    pub pdf_service_endpoint: String,
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

#[derive(Clone, Debug, ValueEnum, Default, PartialEq)]
pub enum Environment {
    #[default]
    Development,
    Production,
    Test,
}
