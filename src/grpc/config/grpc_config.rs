/// gRPC Configuration
///
/// Configuration for gRPC server and actor system
use serde::Deserialize;

/// gRPC server configuration
#[derive(Debug, Clone, Deserialize)]
pub struct GrpcConfig {
    /// Server port
    pub port: u16,

    /// Maximum concurrent connections
    pub max_connections: usize,

    /// Enable server reflection
    pub reflection_enabled: bool,

    /// Enable gRPC-Web
    pub web_enabled: bool,

    /// CORS allowed origins
    pub cors_origins: Vec<String>,

    /// Interceptor configuration
    pub interceptors: InterceptorConfig,
}

/// Interceptor configuration
#[derive(Debug, Clone, Deserialize)]
pub struct InterceptorConfig {
    /// Enable authentication interceptor
    pub auth_enabled: bool,

    /// Enable logging interceptor
    pub logging_enabled: bool,

    /// Enable metrics interceptor
    pub metrics_enabled: bool,

    /// Rate limiting configuration
    pub rate_limiting: Option<RateLimitConfig>,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Deserialize)]
pub struct RateLimitConfig {
    /// Requests per second per client
    pub requests_per_second: u32,

    /// Burst size
    pub burst_size: u32,
}

impl Default for GrpcConfig {
    fn default() -> Self {
        Self {
            port: 50051,
            max_connections: 1000,
            reflection_enabled: true,
            web_enabled: true,
            cors_origins: vec!["*".to_string()],
            interceptors: InterceptorConfig::default(),
        }
    }
}

impl Default for InterceptorConfig {
    fn default() -> Self {
        Self {
            auth_enabled: false,
            logging_enabled: true,
            metrics_enabled: true,
            rate_limiting: None,
        }
    }
}

/// Actor system configuration
#[derive(Debug, Clone, Deserialize)]
pub struct ActorConfig {
    /// User actor pool size
    pub user_pool_size: usize,

    /// Token actor pool size
    pub token_pool_size: usize,

    /// Supervisor strategy
    pub supervisor_strategy: SupervisorStrategy,

    /// Actor mailbox size
    pub mailbox_size: usize,

    /// Restart policy
    pub restart_policy: RestartPolicy,
}

/// Supervisor restart strategy
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SupervisorStrategy {
    /// Restart only the failed actor
    OneForOne,

    /// Restart all actors
    OneForAll,

    /// Restart failed actor and actors started after it
    RestForOne,
}

/// Actor restart policy
#[derive(Debug, Clone, Deserialize)]
pub struct RestartPolicy {
    /// Maximum number of restarts
    pub max_restarts: u32,

    /// Time window for restart counting (seconds)
    pub window_seconds: u64,
}

impl Default for ActorConfig {
    fn default() -> Self {
        Self {
            user_pool_size: 100,
            token_pool_size: 50,
            supervisor_strategy: SupervisorStrategy::OneForOne,
            mailbox_size: 1000,
            restart_policy: RestartPolicy { max_restarts: 3, window_seconds: 60 },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_grpc_config() {
        let config = GrpcConfig::default();
        assert_eq!(config.port, 50051);
        assert_eq!(config.max_connections, 1000);
        assert!(config.reflection_enabled);
    }

    #[test]
    fn test_default_actor_config() {
        let config = ActorConfig::default();
        assert_eq!(config.user_pool_size, 100);
        assert_eq!(config.token_pool_size, 50);
    }
}
