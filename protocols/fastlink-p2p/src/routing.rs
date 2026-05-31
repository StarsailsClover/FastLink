//! Message Routing
//!
//! FastLink-P2P message routing and delivery

use std::collections::{HashMap, VecDeque};
use std::net::SocketAddr;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{debug, warn};
use serde::{Deserialize, Serialize};

use super::dht::{DhtKey, Dht};
use super::node::NodeId;
use super::connection::ConnectionManager;

/// P2P message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum P2PMessageType {
    Ping,
    Pong,
    FindNode(NodeId),
    FoundNode(Vec<NodeInfo>),
    FindValue(DhtKey),
    FoundValue {
        key: DhtKey,
        value: Vec<u8>,
    },
    Store {
        key: DhtKey,
        value: Vec<u8>,
        ttl: u64,
    },
    Relay {
        target: NodeId,
        data: Vec<u8>,
    },
    Application(Vec<u8>),
}

/// Node info for peer discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub node_id: NodeId,
    pub address: SocketAddr,
}

/// P2P message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PMessage {
    pub message_id: u64,
    pub from: NodeId,
    pub to: Option<NodeId>,
    pub message_type: P2PMessageType,
    pub timestamp: u64,
    pub signature: Option<Vec<u8>>,
}

impl P2PMessage {
    /// Create a new P2P message
    pub fn new(
        from: NodeId,
        to: Option<NodeId>,
        message_type: P2PMessageType,
    ) -> Self {
        use rand::Rng;
        
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Self {
            message_id: rand::thread_rng().gen(),
            from,
            to,
            message_type,
            timestamp,
            signature: None,
        }
    }
}

/// Routing table entry
#[derive(Debug, Clone)]
pub struct RoutingEntry {
    pub node_id: NodeId,
    pub address: SocketAddr,
    pub last_seen: u64,
    pub reliability: f64, // 0.0 to 1.0
    pub latency_ms: Option<u64>,
}

/// Message router for P2P messages
pub struct MessageRouter {
    routing_table: Arc<RwLock<HashMap<NodeId, RoutingEntry>>>,
    message_cache: Arc<RwLock<HashMap<u64, P2PMessage>>>,
    max_cache_size: usize,
    message_queue: Arc<RwLock<VecDeque<P2PMessage>>>,
}

#[derive(Debug, Error)]
pub enum RoutingError {
    #[error("Unknown destination: {0}")]
    UnknownDestination(NodeId),
    #[error("Message expired")]
    MessageExpired,
    #[error("Invalid message: {0}")]
    InvalidMessage(String),
}

impl MessageRouter {
    /// Create a new message router
    pub fn new(max_cache_size: usize) -> Self {
        Self {
            routing_table: Arc::new(RwLock::new(HashMap::new())),
            message_cache: Arc::new(RwLock::new(HashMap::new())),
            max_cache_size,
            message_queue: Arc::new(RwLock::new(VecDeque::new())),
        }
    }
    
    /// Route a message to its destination
    pub async fn route(
        &self,
        message: P2PMessage,
        connection_manager: &ConnectionManager,
        dht: &Dht,
    ) -> Result<(), RoutingError> {
        // Cache the message
        self.cache_message(message.clone()).await;
        
        match &message.to {
            Some(target_id) => {
                // Direct message
                if connection_manager.has_connection(target_id).await {
                    // Send directly
                    debug!("Directly routing message to {:?}", target_id);
                    // TODO: Implement direct sending through connection
                } else {
                    // Need to find or relay
                    debug!("Looking up route to {:?}", target_id);
                    let peers = dht.find_closest_peers(&DhtKey(target_id.0[..20].try_into().unwrap()), 5);
                    for peer in peers {
                        // Try to relay through closest peers
                        debug!("Attempting relay through {:?}", peer.node_id);
                    }
                }
            }
            None => {
                // Broadcast message
                debug!("Broadcasting message");
                let all_connections = connection_manager.get_all_info().await;
                for conn_info in all_connections {
                    // TODO: Send to all connected peers
                }
            }
        }
        
        Ok(())
    }
    
    /// Add a message to cache to prevent duplicates
    async fn cache_message(&self, message: P2PMessage) {
        let mut cache = self.message_cache.write().await;
        
        if cache.len() >= self.max_cache_size {
            // Remove oldest messages (simplified)
            let keys: Vec<_> = cache.keys().take(10).cloned().collect();
            for key in keys {
                cache.remove(&key);
            }
        }
        
        cache.insert(message.message_id, message);
    }
    
    /// Check if we've seen a message before
    pub async fn has_seen(&self, message_id: u64) -> bool {
        self.message_cache.read().await.contains_key(&message_id)
    }
    
    /// Update routing table with peer information
    pub async fn update_peer(&self, entry: RoutingEntry) {
        let mut table = self.routing_table.write().await;
        table.insert(entry.node_id, entry);
    }
    
    /// Get a peer from the routing table
    pub async fn get_peer(&self, node_id: &NodeId) -> Option<RoutingEntry> {
        self.routing_table.read().await.get(node_id).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, SocketAddrV4};
    
    #[test]
    fn test_message_creation() {
        let node_id = NodeId::new();
        let message = P2PMessage::new(node_id, None, P2PMessageType::Ping);
        
        assert_eq!(message.from, node_id);
    }
    
    #[test]
    fn test_message_router_creation() {
        let router = MessageRouter::new(1000);
        // Should create successfully
    }
}
