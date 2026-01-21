use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum MailError {
    #[error("Cannot create email message")]
    CannotCreateMessage,

    #[error("Cannot send email message")]
    CannotSendMessage,
}
