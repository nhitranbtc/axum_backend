use super::{CacheError, CacheRepository};
use std::{sync::Arc, time::Duration};
use tracing::debug;

pub struct DistributedLock<T: CacheRepository + ?Sized> {
    repository: Arc<T>,
    key: String,
    value: String,
    ttl: Duration,
}

impl<T: CacheRepository + ?Sized> DistributedLock<T> {
    pub fn new(repository: Arc<T>, key: String, value: String, ttl: Duration) -> Self {
        Self { repository, key, value, ttl }
    }

    pub async fn acquire(&self) -> Result<bool, CacheError> {
        self.repository.set_nx(&self.key, &self.value, self.ttl).await
    }

    pub async fn release(&self) -> Result<(), CacheError> {
        self.repository.delete_if_equals(&self.key, &self.value).await?;
        debug!("Released lock for {}", self.key);
        Ok(())
    }
}
