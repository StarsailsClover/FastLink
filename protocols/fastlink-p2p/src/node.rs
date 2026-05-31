//! P2P Node
//!
//! FastLink P2P node implementation for decentralized communication

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::{RwLock, mpsc};
use tracing::{debug, info, warn};
use libomnilink::IceAgent;
use libomnilink::IceConfig;
use serde::{Deserialize, Serialize};
use libfastcrypto::KeyPair;

/// Node ID - unique identifier for a P2P node
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct NodeId(pub [u8; 32]);

impl NodeId {
    /// Generate a new random NodeId
    pub fn new() -> Self {
        use rand::RngCore;
        let mut bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut bytes);
        NodeId(bytes)
    }
    
    /// Generate a NodeId from a cryptographic key
    pub fn from_keypair(keypair: &KeyPair) -> Self {
        let pub_key = keypair.public_key();
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&pub_key[..32.min(pub_key.len())]);
        NodeId(bytes)
    }
}

impl Default for NodeId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for byte in &self.0[..8] {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

/// P2P Node configuration
#[derive(Debug, Clone)]
pub struct P2PNodeConfig {
    pub node_id: NodeId,
    pub listen_addr: SocketAddr,
    pub bootstrap_nodes: Vec<SocketAddr>,
    pub dht_k_bucket_size: usize,
    pub max_connections: usize,
    pub message_queue_size: usize,
}

impl Default for P2PNodeConfig {
    fn default() -> Self {
        Self {
            node_id: NodeId::new(),
            listen_addr: ([0, 0, 0, 0], 0).into(),
            bootstrap_nodes: Vec::new(),
            dht_k_bucket_size: 20,
            max_connections: 100,
            message_queue_size: 1000,
        }
    }
}

/// P2P Node - main entry point for the FastLink-P2P protocol
pub struct P2PNode {
    config: P2PNodeConfig,
    node_id: NodeId,
    keypair: KeyPair,
    state: RwLock<NodeState>,
    ice_agent: RwLock<Option<IceAgent>>,
    event_sender: mpsc::Sender<P2PEvent>,
    event_receiver: RwLock<Option<mpsc::Receiver<P2PEvent>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeState {
    Idle,
    Bootstrapping,
    Connected,
    Disconnected,
}

#[derive(Debug)]
pub enum P2PEvent {
    PeerConnected(NodeId, SocketAddr),
    PeerDisconnected(NodeId, SocketAddr),
    MessageReceived {
        from: NodeId,
        message: Vec<u8>,
        timestamp: u64,
    },
    NodeError(String),
}

#[derive(Debug, Error)]
pub enum P2PError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("ICE error: {0}")]
    Ice(String),
    #[error("Networking error: {0}")]
    Networking(String),
    #[error("DHT error: {0}")]
    Dht(String),
    #[error("Connection error: {0}")]
    Connection(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
}

impl From<libomnilink::IceError> for P2PError {
    fn from(e: libomnilink::IceError) -> Self {
        Self::Ice(format!("{:?}", e))
    }
}

impl P2PNode {
    /// Create a new P2P node with the given configuration
    pub fn new(config: P2PNodeConfig) -> Self {
        let keypair = KeyPair::generate();
        let node_id = NodeId::from_keypair(&keypair);
        let (event_sender, event_receiver) = mpsc::channel(config.message_queue_size);
        
        Self {
            config,
            node_id,
            keypair,
            state: RwLock::new(NodeState::Idle),
            ice_agent: RwLock::new(None),
            event_sender,
            event_receiver: RwLock::new(Some(event_receiver)),
        }
    }
    
    /// Get the node ID
    pub fn node_id(&self) -> NodeId {
        self.node_id
    }
    
    /// Get a reference to the keypair
    pub fn keypair(&self) -> &KeyPair {
        &self.keypair
    }
    
    /// Start the node and bootstrap to the network
    pub async fn start(&self) -> Result<(), P2PError> {
        info!("Starting FastLink-P2P node {:?}", self.node_id);
        
        *self.state.write().await = NodeState::Bootstrapping;
        
        // Initialize ICE agent for NAT traversal
        let mut ice_agent = IceAgent::new(IceConfig::default());
        ice_agent.start_gathering().await?;
        *self.ice_agent.write().await = Some(ice_agent);
        
        // Start listening for incoming connections
        self.start_listener().await?;
        
        // Bootstrap to the network
        self.bootstrap().await?;
        
        *self.state.write().await = NodeState::Connected;
        
        Ok(())
    }
    
    /// Stop the node
    pub async fn stop(&self) {
        info!("Stopping FastLink-P2P node {:?}", self.node_id);
        
        *self.state.write().await = NodeState::Disconnected;
        
        if let Some(mut ice_agent) = self.ice_agent.write().await.take() {
            ice_agent.close();
        }
    }
    
    /// Start a listener for incoming connections
    async fn start_listener(&self) -> Result<(), P2PError> {
        // TODO: Implement actual TCP/UDP listener with async IO
        info!("P2P node listening on {:?}", self.config.listen_addr);
        
        Ok(())
    }
    
    /// Bootstrap to the P2P network
    async fn bootstrap(&self) -> Result<(), P2PError> {
        for bootstrap_node in &self.config.bootstrap_nodes {
            debug!("Bootstrapping to node: {:?}", bootstrap_node);
            
            // Try to connect to bootstrap node
            if let Err(e) = self.connect(bootstrap_node).await {
                warn!("Failed to connect to bootstrap node {:?}: {:?}", bootstrap_node, e);
            }
        }
        
        Ok(())
    }
    
    /// Connect to a remote peer
    pub async fn connect(&self, addr: &SocketAddr) -> Result<(), P2PError> {
        // TODO: Implement actual peer connection with handshake and authentication
        debug!("Connecting to peer at {:?}", addr);
        
        Ok(())
    }
    
    /// Send a message to a peer
    pub async fn send(&self, target: NodeId, message: Vec<u8>) -> Result<(), P2PError> {
        // TODO: Implement message sending logic
        debug!("Sending message to {:?}", target);
        
        Ok(())
    }
    
    /// Get the event receiver for this node
    pub async fn take_event_receiver(&self) -> Option<mpsc::Receiver<P2PEvent>> {
        self.event_receiver.write().await.take()
    }
    
    /// Subscribe to node events
    pub async fn subscribe(&self) -> mpsc::Receiver<P2PEvent> {
        let (tx, rx) = mpsc::channel(self.config.message_queue_size);
        
        let original_rx = self.take_event_receiver().await;
        
        if let Some(mut original_rx) = original_rx {
            let tx_clone = tx.clone();
            tokio::spawn(async move {
                while let Some(event) = original_rx.recv().await {
                    let _ = tx_clone.send(event).await;
                }
            });
        }
        
        rx
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_node_id_generation() {
        let id1 = NodeId::new();
        let id2 = NodeId::new();
        
        assert_ne!(id1, id2);
    }
    
    #[test]
    fn test_p2p_node_creation() {
        let config = P2PNodeConfig::default();
        let node = P2PNode::new(config);
        
        assert_eq!(*node.state.blocking_read(), NodeState::Idle);
    }
}
