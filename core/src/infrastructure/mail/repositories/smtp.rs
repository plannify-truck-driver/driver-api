use std::sync::Arc;

use lettre::{
    SmtpTransport, Transport,
    message::MessageBuilder,
};
use tera::{Context, Tera};

use crate::{
    domain::{driver::entities::DriverRow, mail::port::MailSmtpRepository},
    infrastructure::mail::repositories::error::MailError,
};
use tracing::error;

#[derive(Clone)]
pub struct SmtpMailRepository {
    mail_client: MessageBuilder,
    transport: SmtpTransport,
    tera: Arc<Tera>,
}

impl SmtpMailRepository {
    pub fn new(mail_client: MessageBuilder, transport: SmtpTransport, tera: Arc<Tera>) -> Self {
        Self {
            mail_client,
            transport,
            tera,
        }
    }
}

impl MailSmtpRepository for SmtpMailRepository {
    fn send_email(&self, to: String, subject: String, body: String) -> Result<(), MailError> {
        let email = self
            .mail_client
            .clone()
            .to(to.parse().unwrap())
            .subject(subject)
            .body(body)
            .map_err(|e| {
                error!("Could not create email content: {:?}", e);
                MailError::CannotCreateMessage
            })?;

        match self.transport.send(&email) {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("Could not send email: {:?}", e);
                Err(MailError::CannotSendMessage)
            }
        }
    }

    async fn send_driver_creation_email(
        &self,
        driver: DriverRow,
        verify_value: String,
        verify_ttl: u64,
    ) -> Result<(), MailError> {
        let mut context = Context::new();
        context.insert("full_name", driver.firstname.as_str());
        context.insert(
            "token_url",
            &format!("https://app.plannify.be/{}", verify_value.as_str()),
        );
        context.insert("duration", &(verify_ttl / 60).to_string());
        context.insert("mail_id", "mail_id_placeholder");

        let template_path = format!("{}/account_verification.html", driver.language.as_str());
        let html_body = self.tera.render(&template_path, &context).map_err(|e| {
            error!("Could not render email template: {:?}", e);
            MailError::CannotCreateMessage
        })?;

        let to = driver.email;

        let subject = match driver.language.as_str() {
            "fr" => "Bienvenue sur Plannify !".to_string(),
            "en" => "Welcome to Plannify!".to_string(),
            _ => "Welcome to Plannify!".to_string(),
        };

        self.send_email(to, subject, html_body)
    }
}
