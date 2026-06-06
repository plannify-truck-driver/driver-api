use redis::{AsyncCommands, aio::ConnectionManager};
use uuid::Uuid;

use crate::{
    domain::mail::{
        entities::{DriverMail, DriverMailPreference, DriverMailType},
        port::{MailCacheKeyType, MailCacheRepository},
    },
    infrastructure::mail::repositories::error::MailError,
};
use tracing::error;

#[derive(Clone)]
pub struct RedisMailCacheRepository {
    connection: ConnectionManager,
}

impl RedisMailCacheRepository {
    pub fn new(connection: ConnectionManager) -> Self {
        Self { connection }
    }
}

impl MailCacheRepository for RedisMailCacheRepository {
    fn generate_redis_key(&self, driver_id: Uuid, suffix: &str) -> String {
        format!("driver:{}:mails:{}", driver_id, suffix)
    }

    #[tracing::instrument(
        name = "cache.mails.get_mails",
        skip(self),
        fields(db.system = "redis", db.operation = "GET", driver_id = %driver_id, page = %page, limit = %limit)
    )]
    async fn get_mails(
        &self,
        driver_id: Uuid,
        page: u32,
        limit: u32,
    ) -> Result<Option<(Vec<DriverMail>, u32)>, MailError> {
        let mut conn = self.connection.clone();
        let (key, _) = self.get_key_by_type(driver_id, MailCacheKeyType::MailsList { page, limit });

        let json: Option<String> = conn.get(key.clone()).await.map_err(|e| {
            error!("Failed to get redis key {}: {:?}", key, e);
            MailError::Internal
        })?;

        let Some(json) = json else {
            return Ok(None);
        };

        match serde_json::from_str(&json) {
            Ok(value) => Ok(Some(value)),
            Err(e) => {
                error!(
                    "Failed to deserialize mails from {}, cache miss: {:?}",
                    key, e
                );
                Ok(None)
            }
        }
    }

    #[tracing::instrument(
        name = "cache.mails.set_mails",
        skip(self, mails),
        fields(db.system = "redis", db.operation = "SET", driver_id = %driver_id, page = %page, limit = %limit)
    )]
    async fn set_mails(
        &self,
        driver_id: Uuid,
        page: u32,
        limit: u32,
        mails: Vec<DriverMail>,
        total: u32,
    ) -> Result<(), MailError> {
        let mut conn = self.connection.clone();
        let (key, ttl) =
            self.get_key_by_type(driver_id, MailCacheKeyType::MailsList { page, limit });

        let json = serde_json::to_string(&(mails, total)).map_err(|e| {
            error!("Failed to serialize mails: {:?}", e);
            MailError::Internal
        })?;

        let _: () = conn.set_ex(key.clone(), json, ttl).await.map_err(|e| {
            error!("Failed to set redis key {}: {:?}", key, e);
            MailError::Internal
        })?;

        Ok(())
    }

    #[tracing::instrument(
        name = "cache.mails.delete_mails",
        skip(self),
        fields(db.system = "redis", db.operation = "DEL", driver_id = %driver_id)
    )]
    async fn delete_mails(&self, driver_id: Uuid) -> Result<(), MailError> {
        let mut conn = self.connection.clone();
        let pattern = format!("driver:{}:mails:list:*", driver_id);
        let keys: Vec<String> = conn.keys(pattern.clone()).await.map_err(|e| {
            error!("Failed to get keys for pattern {}: {:?}", pattern, e);
            MailError::Internal
        })?;

        if !keys.is_empty() {
            let _: () = conn.del(keys).await.map_err(|e| {
                error!("Failed to delete mail keys: {:?}", e);
                MailError::Internal
            })?;
        }

        Ok(())
    }

    #[tracing::instrument(
        name = "cache.mails.get_mail_types",
        skip(self),
        fields(db.system = "redis", db.operation = "GET")
    )]
    async fn get_mail_types(&self) -> Result<Option<Vec<DriverMailType>>, MailError> {
        let mut conn = self.connection.clone();
        let (key, _) = self.mail_types_cache_key();

        let json: Option<String> = conn.get(key.clone()).await.map_err(|e| {
            error!("Failed to get redis key {}: {:?}", key, e);
            MailError::Internal
        })?;

        let Some(json) = json else {
            return Ok(None);
        };

        match serde_json::from_str(&json) {
            Ok(types) => Ok(Some(types)),
            Err(e) => {
                error!(
                    "Failed to deserialize mail types from {}, cache miss: {:?}",
                    key, e
                );
                Ok(None)
            }
        }
    }

    #[tracing::instrument(
        name = "cache.mails.set_mail_types",
        skip(self, types),
        fields(db.system = "redis", db.operation = "SET")
    )]
    async fn set_mail_types(&self, types: Vec<DriverMailType>) -> Result<(), MailError> {
        let mut conn = self.connection.clone();
        let (key, ttl) = self.mail_types_cache_key();

        let json = serde_json::to_string(&types).map_err(|e| {
            error!("Failed to serialize mail types: {:?}", e);
            MailError::Internal
        })?;

        let _: () = conn.set_ex(key.clone(), json, ttl).await.map_err(|e| {
            error!("Failed to set redis key {}: {:?}", key, e);
            MailError::Internal
        })?;

        Ok(())
    }

    #[tracing::instrument(
        name = "cache.mails.get_mail_preferences",
        skip(self),
        fields(db.system = "redis", db.operation = "GET", driver_id = %driver_id)
    )]
    async fn get_mail_preferences(
        &self,
        driver_id: Uuid,
    ) -> Result<Option<Vec<DriverMailPreference>>, MailError> {
        let mut conn = self.connection.clone();
        let (key, _) = self.get_key_by_type(driver_id, MailCacheKeyType::MailPreferences);

        let json: Option<String> = conn.get(key.clone()).await.map_err(|e| {
            error!("Failed to get redis key {}: {:?}", key, e);
            MailError::Internal
        })?;

        let Some(json) = json else {
            return Ok(None);
        };

        match serde_json::from_str(&json) {
            Ok(prefs) => Ok(Some(prefs)),
            Err(e) => {
                error!(
                    "Failed to deserialize preferences from {}, cache miss: {:?}",
                    key, e
                );
                Ok(None)
            }
        }
    }

    #[tracing::instrument(
        name = "cache.mails.set_mail_preferences",
        skip(self, preferences),
        fields(db.system = "redis", db.operation = "SET", driver_id = %driver_id)
    )]
    async fn set_mail_preferences(
        &self,
        driver_id: Uuid,
        preferences: Vec<DriverMailPreference>,
    ) -> Result<(), MailError> {
        let mut conn = self.connection.clone();
        let (key, ttl) = self.get_key_by_type(driver_id, MailCacheKeyType::MailPreferences);

        let json = serde_json::to_string(&preferences).map_err(|e| {
            error!("Failed to serialize preferences: {:?}", e);
            MailError::Internal
        })?;

        let _: () = conn.set_ex(key.clone(), json, ttl).await.map_err(|e| {
            error!("Failed to set redis key {}: {:?}", key, e);
            MailError::Internal
        })?;

        Ok(())
    }

    #[tracing::instrument(
        name = "cache.mails.delete_mail_preferences",
        skip(self),
        fields(db.system = "redis", db.operation = "DEL", driver_id = %driver_id)
    )]
    async fn delete_mail_preferences(&self, driver_id: Uuid) -> Result<(), MailError> {
        let mut conn = self.connection.clone();
        let (key, _) = self.get_key_by_type(driver_id, MailCacheKeyType::MailPreferences);

        let _: () = conn.del(key.clone()).await.map_err(|e| {
            error!("Failed to delete redis key {}: {:?}", key, e);
            MailError::Internal
        })?;

        Ok(())
    }

    #[tracing::instrument(
        name = "cache.mails.get_mail",
        skip(self),
        fields(db.system = "redis", db.operation = "GET", driver_id = %driver_id, mail_id = %mail_id)
    )]
    async fn get_mail(
        &self,
        driver_id: Uuid,
        mail_id: Uuid,
    ) -> Result<Option<DriverMail>, MailError> {
        let mut conn = self.connection.clone();
        let (key, _) = self.get_key_by_type(driver_id, MailCacheKeyType::Mail { mail_id });

        let json: Option<String> = conn.get(key.clone()).await.map_err(|e| {
            error!("Failed to get redis key {}: {:?}", key, e);
            MailError::Internal
        })?;

        let Some(json) = json else {
            return Ok(None);
        };

        match serde_json::from_str(&json) {
            Ok(mail) => Ok(Some(mail)),
            Err(e) => {
                error!(
                    "Failed to deserialize mail from {}, cache miss: {:?}",
                    key, e
                );
                Ok(None)
            }
        }
    }

    #[tracing::instrument(
        name = "cache.mails.set_mail",
        skip(self, mail),
        fields(db.system = "redis", db.operation = "SET", driver_id = %driver_id, mail_id = %mail_id)
    )]
    async fn set_mail(
        &self,
        driver_id: Uuid,
        mail_id: Uuid,
        mail: DriverMail,
    ) -> Result<(), MailError> {
        let mut conn = self.connection.clone();
        let (key, ttl) = self.get_key_by_type(driver_id, MailCacheKeyType::Mail { mail_id });

        let json = serde_json::to_string(&mail).map_err(|e| {
            error!("Failed to serialize mail: {:?}", e);
            MailError::Internal
        })?;

        let _: () = conn.set_ex(key.clone(), json, ttl).await.map_err(|e| {
            error!("Failed to set redis key {}: {:?}", key, e);
            MailError::Internal
        })?;

        Ok(())
    }
}
