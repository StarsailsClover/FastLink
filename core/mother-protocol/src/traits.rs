//! FastLink Core Traits
//!
//! Unified interfaces for all protocol components

use std::net::SocketAddr;
use async_trait::async_trait;
use crate::message::{MessageHeader, MessageType, ProtocolType};
use crate::error::Error;

#[derive(Debug, Clone)]
pub struct NodeId(pub [u8; 32]);

impl NodeId {
    pub fn new(data: [u8; 32]) -> Self {
        Self(data)
    }
    
    pub fn from_public_key(public_key: &[u8]) -> Self {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        hasher.update(public_key);
        let hash = hasher.finalize();
        let mut id = [0u8; 32];
        id.copy_from_slice(hash.as_bytes());
        Self(id)
    }
}

#[derive(Debug, Clone)]
pub struct SessionId(pub u64);

impl SessionId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Closed,
    Connecting,
    Handshake,
    Established,
    Closing,
}

#[derive(Debug, Clone)]
pub struct ConnectionStats {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
    pub latency_ms: f64,
    pub rtt_ms: f64,
}

impl Default for ConnectionStats {
    fn default() -> Self {
        Self {
            bytes_sent: 0,
            bytes_received: 0,
            packets_sent: 0,
            packets_received: 0,
            latency_ms: 0.0,
            rtt_ms: 0.0,
        }
    }
}

#[async_trait]
pub trait Connection: Send + Sync {
    async fn connect(&mut self, addr: SocketAddr) -> Result<(), Error>;
    
    async fn close(&mut self) -> Result<(), Error>;
    
    async fn send(&mut self, data: &[u8]) -> Result<usize, Error>;
    
    async fn recv(&mut self, buf: &mut [u8]) -> Result<usize, Error>;
    
    fn state(&self) -> ConnectionState;
    
    fn local_addr(&self) -> Option<SocketAddr>;
    
    fn peer_addr(&self) -> Option<SocketAddr>;
    
    fn session_id(&self) -> SessionId;
    
    fn stats(&self) -> ConnectionStats;
}

#[async_trait]
pub trait Listener: Send + Sync {
    async fn bind(addr: SocketAddr) -> Result<Self, Error>
    where
        Self: Sized;
    
    async fn accept(&mut self) -> Result<Box<dyn Connection>, Error>;
    
    fn local_addr(&self) -> Option<SocketAddr>;
    
    async fn close(&mut self) -> Result<(), Error>;
}

#[async_trait]
pub trait Node: Send + Sync {
    fn node_id(&self) -> &NodeId;
    
    fn public_key(&self) -> &[u8];
    
    async fn connect_to(&mut self, node_id: &NodeId) -> Result<Box<dyn Connection>, Error>;
    
    async fn listen(&self, addr: SocketAddr) -> Result<Box<dyn Listener>, Error>;
    
    fn local_addr(&self) -> Option<SocketAddr>;
}

pub trait Network: Send + Sync {
    fn send_to(&mut self, node_id: &NodeId, data: &[u8]) -> Result<(), Error>;
    
    fn broadcast(&mut self, data: &[u8]) -> Result<(), Error>;
    
    fn get_connected_nodes(&self) -> Vec<NodeId>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_id_from_public_key() {
        let public_key = b"test_public_key_12345678";
        let node_id = NodeId::from_public_key(public_key);
        
        assert_eq!(node_id.0.len(), 32);
    }

    #[test]
    fn test_session_id() {
        let session = SessionId::new(12345);
        assert_eq!(session.0, 12345);
    }

    #[test]
    fn test_connection_stats_default() {
        let stats = ConnectionStats::default();
        assert_eq!(stats.bytes_sent, 0);
        assert_eq!(stats.bytes_received, 0);
    }
}
