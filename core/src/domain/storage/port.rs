use std::{
    collections::HashMap,
    future::Future,
    sync::{Arc, Mutex},
    time::Duration,
};

use bytes::Bytes;

use crate::infrastructure::storage::repositories::error::StorageError;

pub trait StorageRepository: Send + Sync {
    fn upload(
        &self,
        key: &str,
        data: Bytes,
        content_type: &str,
    ) -> impl Future<Output = Result<(), StorageError>> + Send;

    fn download(&self, key: &str) -> impl Future<Output = Result<Bytes, StorageError>> + Send;

    fn delete(&self, key: &str) -> impl Future<Output = Result<(), StorageError>> + Send;

    fn generate_presigned_url(
        &self,
        key: &str,
        expires_in: Duration,
    ) -> impl Future<Output = Result<String, StorageError>> + Send;
}

#[derive(Clone, Default)]
pub struct MockStorageRepository {
    store: Arc<Mutex<HashMap<String, Bytes>>>,
}

impl MockStorageRepository {
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl StorageRepository for MockStorageRepository {
    async fn upload(
        &self,
        key: &str,
        data: Bytes,
        _content_type: &str,
    ) -> Result<(), StorageError> {
        let mut store = self.store.lock().unwrap();
        store.insert(key.to_string(), data);
        Ok(())
    }

    async fn download(&self, key: &str) -> Result<Bytes, StorageError> {
        let store = self.store.lock().unwrap();
        store.get(key).cloned().ok_or(StorageError::ObjectNotFound)
    }

    async fn delete(&self, key: &str) -> Result<(), StorageError> {
        let mut store = self.store.lock().unwrap();
        store.remove(key);
        Ok(())
    }

    async fn generate_presigned_url(
        &self,
        key: &str,
        _expires_in: Duration,
    ) -> Result<String, StorageError> {
        Ok(format!("http://mock-s3/bucket/{}", key))
    }
}
