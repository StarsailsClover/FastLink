//! FastLink Configuration

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

use crate::error::CommonError;

/// Configuration errors
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),
    
    #[error("Parse error: {0}")]
    Parse(String),
    
    #[error("Missing field: {0}")]
    MissingField(String),
    
    #[error("Invalid value: {0}")]
    InvalidValue(String),
}

/// Main configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub node: NodeConfig,
    pub network: NetworkConfig,
    pub p2p: P2PConfig,
    pub logging: LoggingConfig,
}

impl Config {
    /// Load configuration from file
    pub fn from_file(path: impl Into<PathBuf>) -> Result<Self, ConfigError> {
        let path = path.into();
        let content = std::fs::read_to_string(&path)
            .map_err(|_| ConfigError::FileNotFound(path))?;
        
        let config: Config = toml::from_str(&content)
            .map_err(|e| ConfigError::Parse(e.to_string()))?;
        
        Ok(config)
    }
    
    /// Save configuration to file
    pub fn to_file(&self, path: impl Into<PathBuf>) -> Result<(), ConfigError> {
        let path = path.into();
        let content = toml::to_string_pretty(self)
            .map_err(|e| ConfigError::Parse(e.to_string()))?;
        
        std::fs::write(path, content)
            .map_err(|e| ConfigError::Parse(e.to_string()))?;
        
        Ok(())
    }
}

/// Node configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    pub id: Option<String>,
    pub name: String,
    pub listen_addr: String,
    pub external_addr: Option<String>,
    pub data_dir: PathBuf,
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub max_connections: usize,
    pub connection_timeout: u64,
    pub keepalive_interval: u64,
    pub dht_bootstrap: Vec<String>,
    pub stun_servers: Vec<String>,
}

/// P2P configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PConfig {
    pub nat_traversal: bool,
    pub hole_punch_timeout: u64,
    pub max_relay_hops: u8,
    pub relay_servers: Vec<String>,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub output: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            node: NodeConfig {
                id: None,
                name: "FastLink Node".to_string(),
                listen_addr: "0.0.0.0:0".to_string(),
                external_addr: None,
                data_dir: PathBuf::from("./data"),
            },
            network: NetworkConfig {
                max_connections: 100,
                connection_timeout: 30,
                keepalive_interval: 10,
                dht_bootstrap: vec![],
                stun_servers: vec![],
            },
            p2p: P2PConfig {
                nat_traversal: true,
                hole_punch_timeout: 10,
                max_relay_hops: 3,
                relay_servers: vec![],
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "text".to_string(),
                output: "stdout".to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.node.name, "FastLink Node");
        assert_eq!(config.network.max_connections, 100);
    }
}
