use lettre::{
    SmtpTransport, Transport,
    message::{MessageBuilder, header::ContentType},
};

use crate::{
    domain::{driver::entities::DriverRow, mail::port::MailSmtpRepository},
    infrastructure::mail::repositories::error::MailError,
};
use tracing::error;

#[derive(Clone)]
pub struct SmtpMailRepository {
    mail_client: MessageBuilder,
    transport: SmtpTransport,
}

impl SmtpMailRepository {
    pub fn new(mail_client: MessageBuilder, transport: SmtpTransport) -> Self {
        Self {
            mail_client,
            transport,
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
            .header(ContentType::TEXT_PLAIN)
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

    async fn send_driver_creation_email(&self, driver: DriverRow) -> Result<(), MailError> {
        let to = driver.email;
        let subject = "Welcome to Plannify!".to_string();
        let body = format!(
            "Hello {},\n\nWelcome to Plannify! Your account has been successfully created.\n\nBest regards,\nThe Plannify Team",
            driver.firstname
        );

        self.send_email(to, subject, body)
    }
}
