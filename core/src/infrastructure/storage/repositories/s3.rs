use std::time::Duration;

use aws_sdk_s3::{
    config::{BehaviorVersion, Credentials, Region},
    error::SdkError,
    operation::get_object::GetObjectError,
    presigning::PresigningConfig,
    primitives::ByteStream,
};
use bytes::Bytes;
use tracing::{error, Span};

use crate::{
    domain::storage::port::StorageRepository,
    infrastructure::storage::repositories::error::StorageError,
};

#[derive(Clone)]
pub struct S3StorageRepository {
    client: aws_sdk_s3::Client,
    bucket: String,
}

impl S3StorageRepository {
    pub fn new(
        access_key: &str,
        secret_key: &str,
        endpoint: &str,
        region: &str,
        bucket_name: &str,
    ) -> Self {
        let credentials = Credentials::new(access_key, secret_key, None, None, "Static");

        let config = aws_sdk_s3::config::Builder::new()
            .credentials_provider(credentials)
            .endpoint_url(endpoint)
            .region(Region::new(region.to_string()))
            .force_path_style(true)
            .behavior_version(BehaviorVersion::latest())
            .build();

        Self {
            client: aws_sdk_s3::Client::from_conf(config),
            bucket: bucket_name.to_string(),
        }
    }

    pub fn new_from_client(client: aws_sdk_s3::Client, bucket: String) -> Self {
        Self { client, bucket }
    }
}

impl StorageRepository for S3StorageRepository {
    #[tracing::instrument(
        name = "s3.PutObject",
        skip(self, data),
        fields(
            db.system = "aws.s3",
            db.operation = "PutObject",
            aws.s3.bucket = %self.bucket,
            aws.s3.key = %key,
            aws.s3.content_type = %content_type,
            aws.s3.content_length = tracing::field::Empty,
        )
    )]
    async fn upload(
        &self,
        key: &str,
        data: Bytes,
        content_type: &str,
    ) -> Result<(), StorageError> {
        let content_length = data.len();
        Span::current().record("aws.s3.content_length", content_length);

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(ByteStream::from(data))
            .content_type(content_type)
            .send()
            .await
            .map_err(|e| {
                error!(error = %e, "aws.s3.PutObject failed");
                StorageError::UploadError(e.to_string())
            })?;

        Ok(())
    }

    #[tracing::instrument(
        name = "s3.GetObject",
        skip(self),
        fields(
            db.system = "aws.s3",
            db.operation = "GetObject",
            aws.s3.bucket = %self.bucket,
            aws.s3.key = %key,
            aws.s3.content_length = tracing::field::Empty,
        )
    )]
    async fn download(&self, key: &str) -> Result<Bytes, StorageError> {
        let response = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| match e {
                SdkError::ServiceError(se) => match se.err() {
                    GetObjectError::NoSuchKey(_) => StorageError::ObjectNotFound,
                    _ => {
                        error!("aws.s3.GetObject unexpected service error");
                        StorageError::DownloadError
                    }
                },
                _ => {
                    error!("aws.s3.GetObject failed");
                    StorageError::DownloadError
                }
            })?;

        let data = response
            .body
            .collect()
            .await
            .map_err(|e| {
                error!(error = %e, "aws.s3.GetObject body collection failed");
                StorageError::DownloadError
            })?
            .into_bytes();

        Span::current().record("aws.s3.content_length", data.len());

        Ok(data)
    }

    #[tracing::instrument(
        name = "s3.DeleteObject",
        skip(self),
        fields(
            db.system = "aws.s3",
            db.operation = "DeleteObject",
            aws.s3.bucket = %self.bucket,
            aws.s3.key = %key,
        )
    )]
    async fn delete(&self, key: &str) -> Result<(), StorageError> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| {
                error!(
                    error = %e,
                    "aws.s3.DeleteObject failed"
                );
                StorageError::DeleteError
            })?;

        Ok(())
    }

    #[tracing::instrument(
        name = "s3.PresignGetObject",
        skip(self),
        fields(
            db.system = "aws.s3",
            db.operation = "PresignGetObject",
            aws.s3.bucket = %self.bucket,
            aws.s3.key = %key,
            aws.s3.presign_expires_secs = expires_in.as_secs(),
        )
    )]
    async fn generate_presigned_url(
        &self,
        key: &str,
        expires_in: Duration,
    ) -> Result<String, StorageError> {
        let presigning_config = PresigningConfig::expires_in(expires_in).map_err(|e| {
            error!(error = %e, "aws.s3.PresignGetObject invalid duration");
            StorageError::PresignedUrlError
        })?;

        let presigned = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .presigned(presigning_config)
            .await
            .map_err(|e| {
                error!(
                    error = %e,
                    "aws.s3.PresignGetObject failed"
                );
                StorageError::PresignedUrlError
            })?;

        Ok(presigned.uri().to_string())
    }
}
