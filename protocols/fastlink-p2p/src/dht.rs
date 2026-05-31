//! Distributed Hash Table
//!
//! Kademlia-style distributed hash table for peer discovery and data storage

use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::time::Instant;
use thiserror::Error;
use tracing::{debug, info};
use serde::{Deserialize, Serialize};

use super::node::NodeId;

/// Kademlia K bucket size (20 by default per Kademlia spec)
pub const DEFAULT_K: usize = 20;

/// Maximum DHT value size
pub const MAX_VALUE_SIZE: usize = 65536;

/// DHT key (20 bytes, SHA-1 hash)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DhtKey(pub [u8; 20]);

impl DhtKey {
    /// Generate a DhtKey from a byte slice using SHA-1
    pub fn from_data(data: &[u8]) -> Self {
        use sha1::Digest;
        let hash = sha1::Sha1::digest(data);
        let mut key = [0u8; 20];
        key.copy_from_slice(&hash);
        DhtKey(key)
    }
    
    /// Calculate the XOR distance between two keys
    pub fn distance(&self, other: &DhtKey) -> [u8; 20] {
        let mut result = [0u8; 20];
        for i in 0..20 {
            result[i] = self.0[i] ^ other.0[i];
        }
        result
    }
    
    /// Check if this key is in the same k-bucket as another key
    pub fn bucket_distance(&self, other: &DhtKey) -> usize {
        let xor = self.distance(other);
        
        for i in 0..20 {
            let byte = xor[i];
            if byte == 0 {
                continue;
            }
            
            for bit in 0..8 {
                if (byte >> (7 - bit)) & 1 != 0 {
                    return i * 8 + bit;
                }
            }
        }
        
        159 // same key
    }
}

/// DHT value with expiration time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DhtValue {
    pub data: Vec<u8>,
    pub expiration_time: u64,
    pub publisher: NodeId,
    pub last_published: u64,
}

/// Peer info stored in the DHT
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub node_id: NodeId,
    pub address: SocketAddr,
    pub last_seen: u64, // Unix timestamp in seconds
}

impl PartialEq for PeerInfo {
    fn eq(&self, other: &Self) -> bool {
        self.node_id == other.node_id
    }
}

impl Eq for PeerInfo {}

impl PartialOrd for PeerInfo {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PeerInfo {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.node_id.cmp(&other.node_id)
    }
}

/// Kademlia k-bucket
struct KBucket {
    peers: BTreeMap<NodeId, PeerInfo>,
    k: usize,
}

impl KBucket {
    fn new(k: usize) -> Self {
        Self {
            peers: BTreeMap::new(),
            k,
        }
    }
    
    /// Add a peer to the bucket
    fn add(&mut self, mut peer: PeerInfo) {
        peer.last_seen = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        if self.peers.contains_key(&peer.node_id) {
            // Update existing peer info
            if let Some(existing) = self.peers.get_mut(&peer.node_id) {
                existing.last_seen = peer.last_seen;
                existing.address = peer.address;
            }
        } else if self.peers.len() < self.k {
            // Bucket not full yet, add new peer
            self.peers.insert(peer.node_id, peer);
        }
        // If bucket full, don't add (Kademlia spec)
    }
    
    /// Get peers from the bucket
    fn get_peers(&self, limit: usize) -> Vec<PeerInfo> {
        self.peers.values().cloned().take(limit).collect()
    }
    
    /// Check if the bucket contains a peer
    fn contains(&self, node_id: &NodeId) -> bool {
        self.peers.contains_key(node_id)
    }
}

/// Distributed hash table implementation
pub struct Dht {
    node_id: NodeId,
    k: usize,
    buckets: Vec<KBucket>,
    values: BTreeMap<DhtKey, DhtValue>,
}

#[derive(Debug, Error)]
pub enum DhtError {
    #[error("Invalid key: {0}")]
    InvalidKey(String),
    #[error("Value too large: {0}")]
    ValueTooLarge(usize),
    #[error("Network error: {0}")]
    Network(String),
}

impl Dht {
    /// Create a new DHT
    pub fn new(node_id: NodeId) -> Self {
        Self::new_with_k(node_id, DEFAULT_K)
    }
    
    /// Create a new DHT with custom k bucket size
    pub fn new_with_k(node_id: NodeId, k: usize) -> Self {
        let mut buckets = Vec::with_capacity(160);
        for _ in 0..160 {
            buckets.push(KBucket::new(k));
        }
        
        Self {
            node_id,
            k,
            buckets,
            values: BTreeMap::new(),
        }
    }
    
    /// Add a peer to the DHT
    pub fn add_peer(&mut self, mut peer: PeerInfo) {
        let mut bucket_idx = 159;
        
        // Calculate XOR distance
        for i in 0..32 {
            let xor_byte = self.node_id.0[i] ^ peer.node_id.0[i];
            if xor_byte != 0 {
                // Find the first set bit from the left
                let bit_pos = 7 - xor_byte.leading_zeros() as usize;
                bucket_idx = i * 8 + bit_pos;
                break;
            }
        }
        
        bucket_idx = bucket_idx.min(159);
        self.buckets[bucket_idx].add(peer);
    }
    
    /// Find the k closest peers to a given key
    pub fn find_closest_peers(&self, key: &DhtKey, count: usize) -> Vec<PeerInfo> {
        let mut results = Vec::new();
        
        // Convert DhtKey (20 bytes) to NodeId format (32 bytes) for XOR distance
        let mut key_bytes = [0u8; 32];
        key_bytes[..20].copy_from_slice(&key.0);
        
        // Calculate start bucket
        let mut start_bucket = 159;
        for i in 0..20 { // Only use first 20 bytes
            let xor_byte = self.node_id.0[i] ^ key_bytes[i];
            if xor_byte != 0 {
                let bit_pos = 7 - xor_byte.leading_zeros() as usize;
                start_bucket = i * 8 + bit_pos;
                break;
            }
        }
        start_bucket = start_bucket.min(159);
        
        // Check buckets in order of increasing distance
        for i in start_bucket..160 {
            results.extend(self.buckets[i].get_peers(count - results.len()));
            if results.len() >= count {
                break;
            }
        }
        
        // If not enough, check buckets in reverse order
        if results.len() < count {
            for i in (0..start_bucket).rev() {
                results.extend(self.buckets[i].get_peers(count - results.len()));
                if results.len() >= count {
                    break;
                }
            }
        }
        
        results
    }
    
    /// Store a value in the DHT
    pub fn store(&mut self, key: DhtKey, value: Vec<u8>, publisher: NodeId, ttl: u64) -> Result<(), DhtError> {
        if value.len() > MAX_VALUE_SIZE {
            return Err(DhtError::ValueTooLarge(value.len()));
        }
        
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let dht_value = DhtValue {
            data: value,
            expiration_time: now + ttl,
            publisher,
            last_published: now,
        };
        
        self.values.insert(key, dht_value);
        debug!("Stored value for key {:?}", key);
        
        Ok(())
    }
    
    /// Retrieve a value from the DHT
    pub fn retrieve(&self, key: &DhtKey) -> Option<&DhtValue> {
        let value = self.values.get(key)?;
        
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        if value.expiration_time < now {
            None // expired
        } else {
            Some(value)
        }
    }
    
    /// Check if a peer is known in the DHT
    pub fn is_known_peer(&self, node_id: &NodeId) -> bool {
        for bucket in &self.buckets {
            if bucket.contains(node_id) {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, SocketAddrV4};
    
    #[test]
    fn test_dht_key_from_data() {
        let data = b"hello world";
        let key1 = DhtKey::from_data(data);
        let key2 = DhtKey::from_data(data);
        
        assert_eq!(key1, key2);
    }
    
    #[test]
    fn test_dht_basic_operations() {
        let node_id = NodeId::new();
        let mut dht = Dht::new(node_id);
        
        // Add a peer
        let peer = PeerInfo {
            node_id: NodeId::new(),
            address: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(192, 168, 1, 1), 8080)),
            last_seen: 0,
        };
        
        dht.add_peer(peer);
        
        // Store a value
        let key = DhtKey::from_data(b"test key");
        dht.store(key, b"test value".to_vec(), node_id, 3600).unwrap();
        
        // Retrieve the value
        let retrieved = dht.retrieve(&key);
        assert!(retrieved.is_some());
    }
}
