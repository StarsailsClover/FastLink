//! FastLink DHT Library
//!
//! Distributed Hash Table for node discovery and data storage

use std::collections::{HashMap, VecDeque};
use std::net::SocketAddr;
use std::time::Duration;

use rand::Rng;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, error, info, warn};

use libcommon::time::Timestamp;

/// DHT errors
#[derive(Error, Debug)]
pub enum DhtError {
    #[error("Node not found: {0}")]
    NodeNotFound(String),
    
    #[error("Timeout")]
    Timeout,
    
    #[error("Storage error: {0}")]
    Storage(String),
    
    #[error("Network error: {0}")]
    Network(String),
}

/// Peer information in DHT
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Peer {
    pub id: [u8; 32],
    pub addr: SocketAddr,
    pub last_seen: Timestamp,
    pub reputation: f64,
}

impl Peer {
    pub fn new(id: [u8; 32], addr: SocketAddr) -> Self {
        Self {
            id,
            addr,
            last_seen: Timestamp::now(),
            reputation: 1.0,
        }
    }
}

/// DHT configuration
#[derive(Debug, Clone)]
pub struct DhtConfig {
    pub bucket_size: usize,
    pub replication_factor: usize,
    pub node_timeout: Duration,
    pub query_timeout: Duration,
    pub alpha: usize,  // Concurrent queries
    pub k: usize,      // Bucket size
}

impl Default for DhtConfig {
    fn default() -> Self {
        Self {
            bucket_size: 20,
            replication_factor: 3,
            node_timeout: Duration::from_secs(3600),
            query_timeout: Duration::from_secs(5),
            alpha: 3,
            k: 20,
        }
    }
}

/// Distributed Hash Table
#[derive(Debug)]
pub struct Dht {
    config: DhtConfig,
    store: HashMap<[u8; 32], DhtValue>,
    nodes: VecDeque<Peer>,
    own_id: [u8; 32],
}

/// DHT stored value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DhtValue {
    pub key: [u8; 32],
    pub value: Vec<u8>,
    pub expiration: Timestamp,
    pub publisher: [u8; 32],
}

impl DhtValue {
    pub fn is_expired(&self) -> bool {
        self.expiration.is_before(Timestamp::now())
    }
}

impl Dht {
    /// Create new DHT instance
    pub fn new(config: DhtConfig, own_id: [u8; 32]) -> Self {
        Self {
            config,
            store: HashMap::new(),
            nodes: VecDeque::new(),
            own_id,
        }
    }
    
    /// Store value in DHT
    pub fn store(&mut self, key: [u8; 32], value: Vec<u8>, ttl: Duration) {
        let expiration = Timestamp::now() + ttl;
        let dht_value = DhtValue {
            key,
            value,
            expiration,
            publisher: self.own_id,
        };
        
        self.store.insert(key, dht_value);
        debug!("Stored value for key: {:?}", key);
    }
    
    /// Retrieve value from DHT
    pub fn retrieve(&self, key: &[u8; 32]) -> Option<&DhtValue> {
        self.store.get(key)
    }
    
    /// Remove expired values
    pub fn cleanup_expired(&mut self) {
        let now = Timestamp::now();
        self.store.retain(|_key, value| !value.is_expired());
        
        // Also clean up expired nodes
        self.nodes.retain(|node| {
            let age = now.duration_since(node.last_seen);
            age < self.config.node_timeout
        });
    }
    
    /// Add peer to routing table
    pub fn add_peer(&mut self, peer: Peer) {
        self.nodes.push_back(peer);
        
        // Keep bucket size limited
        while self.nodes.len() > self.config.bucket_size * 10 {
            self.nodes.pop_front();
        }
    }
    
    /// Find closest peers to target
    pub fn find_closest(&self, target: &[u8; 32], count: usize) -> Vec<Peer> {
        let mut peers: Vec<_> = self.nodes.iter().cloned().collect();
        
        peers.sort_by(|a, b| {
            let dist_a = xor_distance(&a.id, target);
            let dist_b = xor_distance(&b.id, target);
            dist_a.cmp(&dist_b)
        });
        
        peers.truncate(count);
        peers
    }
    
    /// Get node count
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
    
    /// Get store size
    pub fn store_size(&self) -> usize {
        self.store.len()
    }
}

/// XOR distance between two node IDs
fn xor_distance(a: &[u8; 32], b: &[u8; 32]) -> [u8; 32] {
    let mut result = [0u8; 32];
    for i in 0..32 {
        result[i] = a[i] ^ b[i];
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_dht_store_retrieve() {
        let config = DhtConfig::default();
        let own_id = [1u8; 32];
        let mut dht = Dht::new(config, own_id);
        
        let key = [2u8; 32];
        let value = b"test_value".to_vec();
        
        dht.store(key, value.clone(), Duration::from_secs(60));
        
        let retrieved = dht.retrieve(&key).unwrap();
        assert_eq!(retrieved.value, value);
    }
    
    #[test]
    fn test_dht_cleanup_expired() {
        let config = DhtConfig::default();
        let own_id = [1u8; 32];
        let mut dht = Dht::new(config, own_id);
        
        // Store with very short TTL
        let key = [3u8; 32];
        dht.store(key, b"value".to_vec(), Duration::from_millis(1));
        
        // Wait for expiration
        std::thread::sleep(Duration::from_millis(10));
        
        dht.cleanup_expired();
        assert!(dht.retrieve(&key).is_none());
    }
    
    #[test]
    fn test_xor_distance() {
        let a = [0b00000000u8; 32];
        let b = [0b11111111u8; 32];
        let dist = xor_distance(&a, &b);
        assert_eq!(dist, [0b11111111u8; 32]);
    }
    
    #[test]
    fn test_find_closest() {
        let config = DhtConfig::default();
        let own_id = [0u8; 32];
        let mut dht = Dht::new(config, own_id);
        
        // Add some peers
        for i in 1..=10u8 {
            let mut id = [0u8; 32];
            id[0] = i;
            let peer = Peer::new(id, "127.0.0.1:0".parse().unwrap());
            dht.add_peer(peer);
        }
        
        let target = [0u8; 32];
        let closest = dht.find_closest(&target, 3);
        
        assert_eq!(closest.len(), 3);
        // Closest should be peers with IDs closest to 0
    }
}
