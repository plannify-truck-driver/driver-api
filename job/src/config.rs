use clap::Parser;
use lettre::SmtpTransport;
use lettre::message::MessageBuilder;
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;

#[derive(Clone, Parser, Debug)]
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

    #[arg(
        long = "frontend-url",
        env = "FRONTEND_URL",
        default_value = "https://app.plannify.be"
    )]
    pub frontend_url: String,

    #[arg(
        long = "pdf-service-endpoint",
        env = "PDF_SERVICE_ENDPOINT",
        default_value = "http://localhost:50051"
    )]
    pub pdf_service_endpoint: String,

    #[command(flatten)]
    pub smtp: SmtpConfig,

    #[command(flatten)]
    pub s3: S3Config,

    #[command(flatten)]
    pub otel: OtelConfig,
}

#[derive(Clone, Parser, Debug)]
pub struct SmtpConfig {
    #[arg(
        long = "smtp-default-sender",
        env = "SMTP_DEFAULT_SENDER",
        default_value = "noreply@plannify.be"
    )]
    pub default_sender: String,

    #[arg(
        long = "smtp-default-sender-reply-to",
        env = "SMTP_DEFAULT_SENDER_REPLY_TO",
        default_value = "noreply@plannify.be"
    )]
    pub default_sender_reply_to: String,

    #[arg(long = "smtp-username", env = "SMTP_USERNAME", default_value = "")]
    pub username: String,

    #[arg(long = "smtp-password", env = "SMTP_PASSWORD", default_value = "")]
    pub password: String,

    #[arg(long = "smtp-domain", env = "SMTP_DOMAIN", default_value = "localhost")]
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
        let creds = Credentials::new(self.username.clone(), self.password.clone());
        SmtpTransport::relay(&self.domain)
            .unwrap_or_else(|_| SmtpTransport::builder_dangerous("localhost"))
            .credentials(creds)
            .build()
    }
}

#[derive(Clone, Parser, Debug)]
pub struct S3Config {
    #[arg(long = "s3-access-key", env = "S3_ACCESS_KEY", default_value = "")]
    pub access_key: String,

    #[arg(long = "s3-secret-key", env = "S3_SECRET_KEY", default_value = "")]
    pub secret_key: String,

    #[arg(
        long = "s3-endpoint",
        env = "S3_ENDPOINT",
        default_value = "http://localhost:3900"
    )]
    pub endpoint: String,

    #[arg(
        long = "s3-bucket-name",
        env = "S3_BUCKET_NAME",
        default_value = "plannify"
    )]
    pub bucket_name: String,

    #[arg(long = "s3-region", env = "S3_REGION", default_value = "garage")]
    pub region: String,
}

#[derive(Clone, Parser, Debug)]
pub struct OtelConfig {
    #[arg(
        long = "otel-exporter-otlp-endpoint",
        env = "OTEL_EXPORTER_OTLP_ENDPOINT",
        default_value = "http://localhost:4317"
    )]
    pub exporter_otlp_endpoint: String,

    #[arg(
        long = "otel-service-name",
        env = "OTEL_SERVICE_NAME",
        default_value = "driver-job"
    )]
    pub service_name: String,
}
