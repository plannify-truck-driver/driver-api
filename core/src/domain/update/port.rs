use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    domain::update::entities::{UpdateCache, UpdateRow},
    infrastructure::update::repositories::error::UpdateError,
};

pub trait UpdateDatabaseRepository: Send + Sync {
    fn get_updates_by_version(
        &self,
        version: String,
        page: u32,
        limit: u32,
    ) -> impl Future<Output = Result<(Vec<UpdateRow>, u32), UpdateError>> + Send;
}

pub trait UpdateCacheRepository: Send + Sync {
    fn generate_redis_key(&self, version: String, page: u32, limit: u32) -> String;

    fn get_updates_by_version(
        &self,
        version: String,
        page: u32,
        limit: u32,
    ) -> impl Future<Output = Result<Option<Vec<UpdateCache>>, UpdateError>> + Send;

    fn set_updates_by_version(
        &self,
        version: String,
        page: u32,
        limit: u32,
        updates: Vec<UpdateCache>,
    ) -> impl Future<Output = Result<(), UpdateError>> + Send;
}

pub trait UpdateService: Send + Sync {
    fn get_updates_by_version(
        &self,
        version: String,
        page: u32,
        limit: u32,
    ) -> impl Future<Output = Result<(Vec<UpdateCache>, u32), UpdateError>> + Send;
}

#[derive(Clone)]
pub struct MockUpdateDatabaseRepository {
    updates: Arc<Mutex<Vec<UpdateRow>>>,
}

impl MockUpdateDatabaseRepository {
    pub fn new() -> Self {
        Self {
            updates: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl Default for MockUpdateDatabaseRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl UpdateDatabaseRepository for MockUpdateDatabaseRepository {
    async fn get_updates_by_version(
        &self,
        version: String,
        page: u32,
        limit: u32,
    ) -> Result<(Vec<UpdateRow>, u32), UpdateError> {
        let updates = self.updates.lock().unwrap();
        let filtered_updates: Vec<UpdateRow> = updates
            .iter()
            .filter(|w| w.version == version)
            .cloned()
            .collect();

        if filtered_updates.len() != 1 {
            return Err(UpdateError::DatabaseError);
        }

        let current_update = &filtered_updates[0];

        let recent_updates: Vec<UpdateRow> = updates
            .iter()
            .filter(|w| w.created_at > current_update.created_at)
            .cloned()
            .collect();

        let result_count = recent_updates.len() as u32;

        let start = ((page - 1) * limit) as usize;
        let end = (start + limit as usize).min(recent_updates.len());
        let paginated_updates = if start < recent_updates.len() {
            recent_updates[start..end].to_vec()
        } else {
            Vec::new()
        };

        Ok((paginated_updates, result_count))
    }
}

type MockUpdateCacheType = HashMap<String, Vec<UpdateCache>>;

#[derive(Clone)]
pub struct MockUpdateCacheRepository {
    updates: Arc<Mutex<MockUpdateCacheType>>,
}

impl MockUpdateCacheRepository {
    pub fn new() -> Self {
        Self {
            updates: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Default for MockUpdateCacheRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl UpdateCacheRepository for MockUpdateCacheRepository {
    fn generate_redis_key(&self, version: String, page: u32, limit: u32) -> String {
        format!("updates:version:{}:page:{}:limit:{}", version, page, limit)
    }

    async fn get_updates_by_version(
        &self,
        version: String,
        page: u32,
        limit: u32,
    ) -> Result<Option<Vec<UpdateCache>>, UpdateError> {
        let updates = self.updates.lock().unwrap();
        let key = self.generate_redis_key(version, page, limit);
        if let Some(cached_updates) = updates.get(&key) {
            Ok(Some(cached_updates.clone()))
        } else {
            Ok(None)
        }
    }

    async fn set_updates_by_version(
        &self,
        version: String,
        page: u32,
        limit: u32,
        updates: Vec<UpdateCache>,
    ) -> Result<(), UpdateError> {
        let mut cache = self.updates.lock().unwrap();
        let key = self.generate_redis_key(version, page, limit);
        cache.insert(key, updates);
        Ok(())
    }
}
