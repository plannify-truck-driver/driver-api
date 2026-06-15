use std::sync::Arc;

use lettre::{SmtpTransport, Transport, message::MessageBuilder};
use tera::{Context, Tera};

use crate::{
    domain::{driver::entities::DriverRow, mail::port::MailSmtpRepository},
    infrastructure::mail::repositories::error::MailError,
};
use tracing::{error, warn};

#[derive(Clone)]
pub struct SmtpMailRepository {
    mail_client: MessageBuilder,
    transport: SmtpTransport,
    tera: Arc<Tera>,
    frontend_url: String,
    is_test_environment: bool,
}

impl SmtpMailRepository {
    pub fn new(
        mail_client: MessageBuilder,
        transport: SmtpTransport,
        tera: Arc<Tera>,
        frontend_url: String,
        is_test_environment: bool,
    ) -> Self {
        Self {
            mail_client,
            transport,
            tera,
            frontend_url,
            is_test_environment,
        }
    }
}

impl MailSmtpRepository for SmtpMailRepository {
    #[tracing::instrument(
        name = "smtp.mails.send_email",
        skip(self),
        fields(
            to = ?to,
            subject = ?subject,
        )
    )]
    fn send_email(&self, to: String, subject: String, body: String) -> Result<(), MailError> {
        if self.is_test_environment {
            warn!(
                "Test Environment: Email to {} with subject '{}' not sent.",
                to, subject
            );
            return Ok(());
        }

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

    #[tracing::instrument(
        name = "smtp.mails.send_driver_deactivation_email",
        skip(self),
        fields(driver_id = %driver.pk_driver_id)
    )]
    async fn send_driver_deactivation_email(&self, driver: DriverRow) -> Result<(), MailError> {
        if self.is_test_environment {
            warn!(
                "Test Environment: Driver deactivation email to {} not sent.",
                driver.email
            );
            return Ok(());
        }

        let deactivation_date = driver
            .deactivated_at
            .map(|dt| {
                let fmt = match driver.language.as_str() {
                    "fr" => "%d/%m/%Y",
                    _ => "%m/%d/%Y",
                };
                dt.format(fmt).to_string()
            })
            .unwrap_or_default();

        let mut context = Context::new();
        context.insert("full_name", driver.firstname.as_str());
        context.insert("deactivation_date", &deactivation_date);

        let template_path = format!("{}/account_deactivation.html", driver.language.as_str());
        let html_body = self.tera.render(&template_path, &context).map_err(|e| {
            error!("Could not render email template: {:?}", e);
            MailError::CannotCreateMessage
        })?;

        let subject = match driver.language.as_str() {
            "fr" => "Désactivation de votre compte Plannify".to_string(),
            "en" => "Your Plannify account has been deactivated".to_string(),
            _ => {
                error!("Unsupported driver language: {}", driver.language);
                return Err(MailError::Internal);
            }
        };

        self.send_email(driver.email, subject, html_body)
    }

    #[tracing::instrument(
        name = "smtp.mails.send_driver_reactivation_email",
        skip(self),
        fields(driver_id = %driver.pk_driver_id)
    )]
    async fn send_driver_reactivation_email(&self, driver: DriverRow) -> Result<(), MailError> {
        if self.is_test_environment {
            warn!(
                "Test Environment: Driver reactivation email to {} not sent.",
                driver.email
            );
            return Ok(());
        }

        let mut context = Context::new();
        context.insert("full_name", driver.firstname.as_str());

        let template_path = format!("{}/account_reactivation.html", driver.language.as_str());
        let html_body = self.tera.render(&template_path, &context).map_err(|e| {
            error!("Could not render email template: {:?}", e);
            MailError::CannotCreateMessage
        })?;

        let subject = match driver.language.as_str() {
            "fr" => "Réactivation de votre compte Plannify".to_string(),
            "en" => "Your Plannify account has been reactivated".to_string(),
            _ => {
                error!("Unsupported driver language: {}", driver.language);
                return Err(MailError::Internal);
            }
        };

        self.send_email(driver.email, subject, html_body)
    }

    #[tracing::instrument(
        name = "smtp.mails.send_driver_reset_password_email",
        skip(self),
        fields(driver_id = %driver.pk_driver_id)
    )]
    async fn send_driver_reset_password_email(
        &self,
        driver: DriverRow,
        reset_value: String,
        reset_ttl: u64,
    ) -> Result<(), MailError> {
        if self.is_test_environment {
            warn!(
                "Test Environment: Driver reset password email to {} not sent.",
                driver.email
            );
            return Ok(());
        }

        let mut context = Context::new();
        context.insert("full_name", driver.firstname.as_str());
        context.insert(
            "token_url",
            &format!(
                "{}/authentication/reset-password?token={}&id={}",
                self.frontend_url,
                reset_value.as_str(),
                driver.pk_driver_id
            ),
        );
        context.insert("duration", &(reset_ttl / 60).to_string());

        let template_path = format!("{}/password_reset.html", driver.language.as_str());
        let html_body = self.tera.render(&template_path, &context).map_err(|e| {
            error!("Could not render email template: {:?}", e);
            MailError::CannotCreateMessage
        })?;

        let subject = match driver.language.as_str() {
            "fr" => "Réinitialisation de votre mot de passe Plannify".to_string(),
            "en" => "Reset your Plannify password".to_string(),
            _ => {
                error!("Unsupported driver language: {}", driver.language);
                return Err(MailError::Internal);
            }
        };

        self.send_email(driver.email, subject, html_body)
    }

    #[tracing::instrument(
        name = "smtp.mails.send_driver_creation_email",
        skip(self),
        fields(
            driver_id = %driver.pk_driver_id,
        )
    )]
    async fn send_driver_creation_email(
        &self,
        driver: DriverRow,
        verify_value: String,
        verify_ttl: u64,
    ) -> Result<(), MailError> {
        if self.is_test_environment {
            warn!(
                "Test Environment: Driver creation email to {} not sent.",
                driver.email
            );
            return Ok(());
        }

        let mut context = Context::new();
        context.insert("full_name", driver.firstname.as_str());
        context.insert(
            "token_url",
            &format!(
                "{}/authentication/token/verify-account?token={}&id={}",
                self.frontend_url,
                verify_value.as_str(),
                driver.pk_driver_id
            ),
        );
        context.insert("duration", &(verify_ttl / 60).to_string());

        let template_path = format!("{}/account_creation.html", driver.language.as_str());
        let html_body = self.tera.render(&template_path, &context).map_err(|e| {
            error!("Could not render email template: {:?}", e);
            MailError::CannotCreateMessage
        })?;

        let to = driver.email;

        let subject = match driver.language.as_str() {
            "fr" => "Bienvenue sur Plannify !".to_string(),
            "en" => "Welcome to Plannify!".to_string(),
            _ => {
                error!("Unsupported driver language: {}", driver.language);
                return Err(MailError::Internal);
            }
        };

        self.send_email(to, subject, html_body)
    }

    async fn send_driver_verification_email(
        &self,
        driver: DriverRow,
        verify_value: String,
        verify_ttl: u64,
    ) -> Result<(), MailError>
    {
        if self.is_test_environment {
            warn!(
                "Test Environment: Driver verification email to {} not sent.",
                driver.email
            );
            return Ok(());
        }

        let mut context = Context::new();
        context.insert("full_name", driver.firstname.as_str());
        context.insert(
            "token_url",
            &format!(
                "{}/authentication/token/verify-account?token={}&id={}",
                self.frontend_url,
                verify_value.as_str(),
                driver.pk_driver_id
            ),
        );
        context.insert("duration", &(verify_ttl / 60).to_string());

        let template_path = format!("{}/verify_account.html", driver.language.as_str());
        let html_body = self.tera.render(&template_path, &context).map_err(|e| {
            error!("Could not render email template: {:?}", e);
            MailError::CannotCreateMessage
        })?;

        let to = driver.email;

        let subject = match driver.language.as_str() {
            "fr" => "Bienvenue sur Plannify !".to_string(),
            "en" => "Welcome to Plannify!".to_string(),
            _ => {
                error!("Unsupported driver language: {}", driver.language);
                return Err(MailError::Internal);
            }
        };

        self.send_email(to, subject, html_body)
    }
}
