use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum MailError {
    #[error("Internal server error")]
    Internal,

    #[error("Cannot create email message")]
    CannotCreateMessage,

    #[error("Cannot send email message")]
    CannotSendMessage,

    #[error("Database error")]
    DatabaseError,

    #[error("Mail not found")]
    MailNotFound,

    #[error("Mail type not found")]
    MailTypeNotFound,
}
