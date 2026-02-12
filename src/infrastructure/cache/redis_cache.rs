use super::{CacheError, CacheRepository};
use async_trait::async_trait;
use redis::{aio::ConnectionManager, AsyncCommands};
use std::time::Duration;
use tracing::{debug, error};

#[derive(Clone)]
pub struct RedisCacheRepository {
    connection_manager: ConnectionManager,
}

impl RedisCacheRepository {
    pub async fn new(redis_url: &str) -> Result<Self, CacheError> {
        let client = redis::Client::open(redis_url)?;
        let connection_manager = client
            .get_connection_manager()
            .await
            .map_err(|e| CacheError::Connection(e.to_string()))?;

        Ok(Self { connection_manager })
    }
}

#[async_trait]
impl CacheRepository for RedisCacheRepository {
    async fn get(&self, key: &str) -> Result<Option<String>, CacheError> {
        let mut conn = self.connection_manager.clone();
        match conn.get(key).await {
            Ok(value) => {
                debug!("Cache hit for key: {}", key);
                Ok(value)
            },
            Err(e) => {
                error!("Failed to get cache key {}: {}", key, e);
                Err(CacheError::Redis(e))
            },
        }
    }

    async fn set(&self, key: &str, value: &str, ttl: Duration) -> Result<(), CacheError> {
        let mut conn = self.connection_manager.clone();
        let ttl_seconds = ttl.as_secs() as usize;

        // Use SETEX for atomic set with expiry
        match conn.set_ex::<_, _, ()>(key, value, ttl_seconds as u64).await {
            Ok(_) => {
                debug!("Cache set for key: {} (TTL: {}s)", key, ttl_seconds);
                Ok(())
            },
            Err(e) => {
                error!("Failed to set cache key {}: {}", key, e);
                Err(CacheError::Redis(e))
            },
        }
    }

    async fn delete(&self, key: &str) -> Result<(), CacheError> {
        let mut conn = self.connection_manager.clone();
        match conn.del(key).await {
            Ok(()) => {
                debug!("Cache deleted for key: {}", key);
                Ok(())
            },
            Err(e) => {
                error!("Failed to delete cache key {}: {}", key, e);
                Err(CacheError::Redis(e))
            },
        }
    }

    async fn set_nx(&self, key: &str, value: &str, ttl: Duration) -> Result<bool, CacheError> {
        let mut conn = self.connection_manager.clone();
        let ttl_millis = ttl.as_millis() as u64;

        // Redis SET key value PX ttl NX
        // Returns "OK" if set, or nil if not set
        let result: Option<String> = redis::cmd("SET")
            .arg(key)
            .arg(value)
            .arg("PX")
            .arg(ttl_millis)
            .arg("NX")
            .query_async(&mut conn)
            .await
            .map_err(CacheError::Redis)?;

        Ok(result.is_some())
    }

    async fn delete_if_equals(&self, key: &str, value: &str) -> Result<bool, CacheError> {
        let script = r#"
            if redis.call("get", KEYS[1]) == ARGV[1] then
                return redis.call("del", KEYS[1])
            else
                return 0
            end
        "#;
        match self.eval_script(script, vec![key], vec![value]).await {
            Ok(v) => {
                let count: i64 = redis::from_redis_value(v).unwrap_or(0);
                Ok(count == 1)
            },
            Err(e) => Err(e),
        }
    }
}

impl RedisCacheRepository {
    pub async fn eval_script<K, A>(
        &self,
        script: &str,
        keys: K,
        args: A,
    ) -> Result<redis::Value, CacheError>
    where
        K: redis::ToRedisArgs + Send + Sync,
        A: redis::ToRedisArgs + Send + Sync,
    {
        let mut conn = self.connection_manager.clone();
        let script = redis::Script::new(script);

        script
            .key(keys)
            .arg(args)
            .invoke_async(&mut conn)
            .await
            .map_err(CacheError::Redis)
    }
}
