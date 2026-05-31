//! Peer Discovery
//!
//! Peer discovery mechanisms for FastLink-P2P

use std::collections::HashSet;
use std::net::{SocketAddr, Ipv4Addr, Ipv6Addr};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use serde::{Deserialize, Serialize};

use super::node::NodeId;
use super::dht::{Dht, PeerInfo, DhtKey};

/// Discovery method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DiscoveryMethod {
    Bootstrap,
    Dht,
    Multicast,
    Manual,
}

/// Peer discovery configuration
#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    pub bootstrap_nodes: Vec<SocketAddr>,
    pub enable_multicast: bool,
    pub multicast_addresses: Vec<SocketAddr>,
    pub discovery_interval_seconds: u64,
    pub max_peers_to_find: usize,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            bootstrap_nodes: Vec::new(),
            enable_multicast: true,
            multicast_addresses: vec![
                SocketAddr::new(Ipv4Addr::new(224, 0, 0, 251).into(), 5353),
                SocketAddr::new(Ipv6Addr::new(0xff02, 0, 0, 0, 0, 0, 0, 0xfb).into(), 5353),
            ],
            discovery_interval_seconds: 60,
            max_peers_to_find: 50,
        }
    }
}

/// Discovered peer
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct DiscoveredPeer {
    pub node_id: NodeId,
    pub address: SocketAddr,
    pub discovery_method: DiscoveryMethod,
    pub discovered_at: u64,
    pub last_seen: Option<u64>,
    pub version: Option<String>,
}

/// Peer discovery service
pub struct PeerDiscovery {
    config: DiscoveryConfig,
    discovered_peers: Arc<RwLock<HashSet<DiscoveredPeer>>>,
    dht: Arc<RwLock<Option<Dht>>>,
}

#[derive(Debug, Error)]
pub enum DiscoveryError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Network error: {0}")]
    Network(String),
    #[error("DHT error: {0}")]
    Dht(String),
}

impl PeerDiscovery {
    /// Create a new peer discovery service
    pub fn new(config: DiscoveryConfig) -> Self {
        Self {
            config,
            discovered_peers: Arc::new(RwLock::new(HashSet::new())),
            dht: Arc::new(RwLock::new(None)),
        }
    }
    
    /// Set the DHT reference for discovery
    pub async fn set_dht(&self, dht: Dht) {
        *self.dht.write().await = Some(dht);
    }
    
    /// Start the discovery service
    pub async fn start(&self) -> Result<(), DiscoveryError> {
        info!("Starting peer discovery service");
        
        // Bootstrap discovery
        self.bootstrap_discovery().await?;
        
        // Start periodic discovery
        if self.config.enable_multicast {
            self.start_multicast_discovery().await?;
        }
        
        Ok(())
    }
    
    /// Perform bootstrap discovery by connecting to known nodes
    async fn bootstrap_discovery(&self) -> Result<(), DiscoveryError> {
        if self.config.bootstrap_nodes.is_empty() {
            debug!("No bootstrap nodes configured");
            return Ok(());
        }
        
        info!("Bootstrapping from {:?} nodes", self.config.bootstrap_nodes.len());
        
        for addr in &self.config.bootstrap_nodes {
            if let Err(e) = self.discover_from_bootstrap(addr).await {
                warn!("Failed to bootstrap from {:?}: {:?}", addr, e);
            }
        }
        
        Ok(())
    }
    
    /// Discover peers from a bootstrap node
    async fn discover_from_bootstrap(&self, addr: &SocketAddr) -> Result<Vec<DiscoveredPeer>, DiscoveryError> {
        debug!("Discovering peers from bootstrap node: {:?}", addr);
        
        // TODO: Implement actual bootstrap protocol
        // For now, return empty list
        Ok(Vec::new())
    }
    
    /// Start multicast peer discovery
    async fn start_multicast_discovery(&self) -> Result<(), DiscoveryError> {
        info!("Starting multicast discovery");
        
        for addr in &self.config.multicast_addresses {
            debug!("Joining multicast group: {:?}", addr);
            // TODO: Implement actual multicast listening
        }
        
        Ok(())
    }
    
    /// Add a manually discovered peer
    pub async fn add_peer(&self, node_id: NodeId, addr: SocketAddr) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let peer = DiscoveredPeer {
            node_id,
            address: addr,
            discovery_method: DiscoveryMethod::Manual,
            discovered_at: now,
            last_seen: Some(now),
            version: None,
        };
        
        self.discovered_peers.write().await.insert(peer);
    }
    
    /// Get all discovered peers
    pub async fn get_discovered_peers(&self) -> Vec<DiscoveredPeer> {
        self.discovered_peers.read().await.iter().cloned().collect()
    }
    
    /// Find peers using the DHT
    pub async fn find_peers_dht(&self, key: &DhtKey, count: usize) -> Result<Vec<PeerInfo>, DiscoveryError> {
        if let Some(dht) = self.dht.read().await.as_ref() {
            Ok(dht.find_closest_peers(key, count))
        } else {
            Ok(Vec::new())
        }
    }
    
    /// Check if we've already discovered a peer
    pub async fn has_discovered(&self, node_id: &NodeId) -> bool {
        self.discovered_peers.read().await.iter()
            .any(|p| p.node_id == *node_id)
    }
}

/// LAN multicast beacon for local peer discovery
pub struct MulticastBeacon {
    local_node_id: NodeId,
    port: u16,
    running: Arc<RwLock<bool>>,
}

impl MulticastBeacon {
    /// Create a new multicast beacon
    pub fn new(local_node_id: NodeId, port: u16) -> Self {
        Self {
            local_node_id,
            port,
            running: Arc::new(RwLock::new(false)),
        }
    }
    
    /// Start broadcasting beacons
    pub async fn start(&self) -> Result<(), DiscoveryError> {
        *self.running.write().await = true;
        info!("Multicast beacon started on port {}", self.port);
        
        // TODO: Implement actual beacon broadcasting
        Ok(())
    }
    
    /// Stop broadcasting beacons
    pub async fn stop(&self) {
        *self.running.write().await = false;
        info!("Multicast beacon stopped");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, SocketAddrV4};
    
    #[test]
    fn test_discovery_config_default() {
        let config = DiscoveryConfig::default();
        assert!(config.enable_multicast);
    }
    
    #[test]
    fn test_peer_discovery_creation() {
        let config = DiscoveryConfig::default();
        let discovery = PeerDiscovery::new(config);
        // Should create successfully
    }
    
    #[test]
    fn test_add_peer() {
        let config = DiscoveryConfig::default();
        let discovery = PeerDiscovery::new(config);
        
        let node_id = NodeId::new();
        let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080));
        
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            discovery.add_peer(node_id, addr).await;
            assert!(discovery.has_discovered(&node_id).await);
        });
    }
}
