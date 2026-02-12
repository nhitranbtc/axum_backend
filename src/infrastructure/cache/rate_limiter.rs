use super::{redis_cache::RedisCacheRepository, CacheError};
use tracing::{debug, error};

pub struct RateLimiter {
    repository: RedisCacheRepository,
}

impl RateLimiter {
    pub fn new(repository: RedisCacheRepository) -> Self {
        Self { repository }
    }

    /// Check if a request is allowed based on rate limits
    /// Uses a sliding window log algorithm via Lua script
    /// Key: Identifier (e.g., IP or UserID)
    /// Limit: Max requests allowed
    /// Window: Time window in seconds
    pub async fn check(&self, key: &str, limit: usize, window: u64) -> Result<bool, CacheError> {
        let rate_limit_key = format!("ratelimit:{}", key);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let window_ms = window * 1000;
        let clear_before = now - window_ms;

        // Lua script for atomic sliding window rate limiting
        // ARGV[1]: current timestamp (ms)
        // ARGV[2]: clear entries before this timestamp (ms)
        // ARGV[3]: window duration (ms) - to set TTL
        // ARGV[4]: limit
        let script = r#"
            -- Remove old entries
            redis.call('ZREMRANGEBYSCORE', KEYS[1], 0, ARGV[2])
            
            -- Count current entries
            let count = redis.call('ZCARD', KEYS[1])
            
            if count < tonumber(ARGV[4]) then
                -- Add new entry
                redis.call('ZADD', KEYS[1], ARGV[1], ARGV[1])
                redis.call('PEXPIRE', KEYS[1], ARGV[3])
                return 1 -- Allowed
            else
                return 0 -- Denied
            end
        "#;

        match self
            .repository
            .eval_script(
                script,
                vec![&rate_limit_key],
                vec![
                    now.to_string().as_str(),
                    clear_before.to_string().as_str(),
                    window_ms.to_string().as_str(),
                    limit.to_string().as_str(),
                ],
            )
            .await
        {
            Ok(val) => {
                let allowed: i64 = redis::from_redis_value(val).unwrap_or(0);
                if allowed == 1 {
                    debug!("Rate limit check passed for {}", key);
                    Ok(true)
                } else {
                    debug!("Rate limit exceeded for {}", key);
                    Ok(false)
                }
            },
            Err(e) => {
                error!("Rate limit check error for {}: {}", key, e);
                // Fail open or closed? Here failing closed (deny) for safety, or open (allow) for availability
                // Let's return error and let caller decide
                Err(e)
            },
        }
    }
}
