use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum StorageError {
    #[error("Object not found")]
    ObjectNotFound,

    #[error("Failed to upload object")]
    UploadError,

    #[error("Failed to download object")]
    DownloadError,

    #[error("Failed to delete object")]
    DeleteError,

    #[error("Failed to generate presigned URL")]
    PresignedUrlError,

    #[error("An internal error occurred")]
    Internal,
}
