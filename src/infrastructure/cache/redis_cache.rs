use super::{CacheError, CacheRepository};
use async_trait::async_trait;
use redis::{aio::ConnectionManager, cluster_async::ClusterConnection, AsyncCommands};
use std::time::Duration;
use tracing::{debug, error};

#[derive(Clone)]
enum RedisConnection {
    Single(ConnectionManager),
    Cluster(ClusterConnection),
}

#[derive(Clone)]
pub struct RedisCacheRepository {
    connection: RedisConnection,
}

impl RedisCacheRepository {
    pub async fn new(redis_url: &str) -> Result<Self, CacheError> {
        if redis_url.starts_with("redis-cluster://") {
            let nodes: Vec<&str> = redis_url
                .trim_start_matches("redis-cluster://")
                .split(',')
                .collect();

            let client = redis::cluster::ClusterClient::new(nodes)
                .map_err(CacheError::Redis)?;
            
            let connection = client
                .get_async_connection()
                .await
                .map_err(|e| CacheError::Connection(e.to_string()))?;

            Ok(Self {
                connection: RedisConnection::Cluster(connection),
            })
        } else {
            let client = redis::Client::open(redis_url)?;
            let connection_manager = client
                .get_connection_manager()
                .await
                .map_err(|e| CacheError::Connection(e.to_string()))?;

            Ok(Self {
                connection: RedisConnection::Single(connection_manager),
            })
        }
    }

    pub async fn eval_script<K, A>(
        &self,
        script: &str,
        keys: K,
        args: A,
    ) -> Result<redis::Value, CacheError>
    where
        K: redis::ToRedisArgs + Send + Sync + Clone,
        A: redis::ToRedisArgs + Send + Sync + Clone,
    {
        let script = redis::Script::new(script);
        match &self.connection {
            RedisConnection::Single(conn) => {
                let mut conn = conn.clone();
                script
                    .key(keys)
                    .arg(args)
                    .invoke_async(&mut conn)
                    .await
                    .map_err(CacheError::Redis)
            }
            RedisConnection::Cluster(conn) => {
                let mut conn = conn.clone();
                script
                    .key(keys)
                    .arg(args)
                    .invoke_async(&mut conn)
                    .await
                    .map_err(CacheError::Redis)
            }
        }
    }
}

#[async_trait]
impl CacheRepository for RedisCacheRepository {
    async fn get(&self, key: &str) -> Result<Option<String>, CacheError> {
        let result = match &self.connection {
            RedisConnection::Single(conn) => {
                let mut conn = conn.clone();
                conn.get(key).await
            }
            RedisConnection::Cluster(conn) => {
                let mut conn = conn.clone();
                conn.get(key).await
            }
        };

        match result {
            Ok(value) => {
                debug!("Cache hit for key: {}", key);
                Ok(value)
            }
            Err(e) => {
                error!("Failed to get cache key {}: {}", key, e);
                Err(CacheError::Redis(e))
            }
        }
    }

    async fn set(&self, key: &str, value: &str, ttl: Duration) -> Result<(), CacheError> {
        let ttl_seconds = ttl.as_secs();
        let result = match &self.connection {
            RedisConnection::Single(conn) => {
                let mut conn = conn.clone();
                conn.set_ex::<_, _, ()>(key, value, ttl_seconds).await
            }
            RedisConnection::Cluster(conn) => {
                let mut conn = conn.clone();
                conn.set_ex::<_, _, ()>(key, value, ttl_seconds).await
            }
        };

        match result {
            Ok(_) => {
                debug!("Cache set for key: {} (TTL: {}s)", key, ttl_seconds);
                Ok(())
            }
            Err(e) => {
                error!("Failed to set cache key {}: {}", key, e);
                Err(CacheError::Redis(e))
            }
        }
    }

    async fn delete(&self, key: &str) -> Result<(), CacheError> {
        let result = match &self.connection {
            RedisConnection::Single(conn) => {
                let mut conn = conn.clone();
                conn.del(key).await
            }
            RedisConnection::Cluster(conn) => {
                let mut conn = conn.clone();
                conn.del(key).await
            }
        };

        match result {
            Ok(()) => {
                debug!("Cache deleted for key: {}", key);
                Ok(())
            }
            Err(e) => {
                error!("Failed to delete cache key {}: {}", key, e);
                Err(CacheError::Redis(e))
            }
        }
    }

    async fn set_nx(&self, key: &str, value: &str, ttl: Duration) -> Result<bool, CacheError> {
        let ttl_millis = ttl.as_millis() as u64;
        let mut cmd = redis::cmd("SET");
        cmd.arg(key)
            .arg(value)
            .arg("PX")
            .arg(ttl_millis)
            .arg("NX");

        let result: Result<Option<String>, redis::RedisError> = match &self.connection {
            RedisConnection::Single(conn) => {
                let mut conn = conn.clone();
                cmd.query_async(&mut conn).await
            }
            RedisConnection::Cluster(conn) => {
                let mut conn = conn.clone();
                cmd.query_async(&mut conn).await
            }
        };

        match result {
            Ok(v) => Ok(v.is_some()),
            Err(e) => Err(CacheError::Redis(e)),
        }
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
            }
            Err(e) => Err(e),
        }
    }
}
