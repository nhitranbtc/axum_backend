use serde::Deserialize;
use std::env;

/// Configuration for ScyllaDB connection
#[derive(Debug, Clone, Deserialize)]
pub struct ScyllaConfig {
    /// Comma-separated list of ScyllaDB nodes (e.g., "localhost:9042,node2:9042")
    pub nodes: Vec<String>,
    
    /// Keyspace name
    pub keyspace: String,
    
    /// Optional username for authentication
    pub username: Option<String>,
    
    /// Optional password for authentication
    pub password: Option<String>,
    
    /// Replication factor for the keyspace
    pub replication_factor: usize,
}

impl ScyllaConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> anyhow::Result<Self> {
        let nodes_str = env::var("SCYLLA_NODES")
            .map_err(|_| anyhow::anyhow!("SCYLLA_NODES must be set"))?;
        
        let nodes = nodes_str
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
        
        let keyspace = env::var("SCYLLA_KEYSPACE")
            .map_err(|_| anyhow::anyhow!("SCYLLA_KEYSPACE must be set"))?;
        
        let username = env::var("SCYLLA_USERNAME").ok();
        let password = env::var("SCYLLA_PASSWORD").ok();
        
        let replication_factor = env::var("SCYLLA_REPLICATION_FACTOR")
            .map_err(|_| anyhow::anyhow!("SCYLLA_REPLICATION_FACTOR must be set"))?
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid SCYLLA_REPLICATION_FACTOR: {}", e))?;
        
        Ok(Self {
            nodes,
            keyspace,
            username,
            password,
            replication_factor,
        })
    }
}
