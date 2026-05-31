//! FastLink Key Manager Module
//!
//! Key management for session keys and rotating keys

use dashmap::DashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyInfo {
    pub key_id: u64,
    pub key: Vec<u8>,
    pub created_at: u64,
    pub expires_at: u64,
}

pub struct KeyManager {
    current_key: Arc<RwLock<Option<KeyInfo>>>,
    next_key: Arc<RwLock<Option<KeyInfo>>>,
    previous_keys: Arc<DashMap<u64, KeyInfo>>,
    rotation_interval: u64,
}

impl KeyManager {
    pub fn new(rotation_interval: u64) -> Self {
        Self {
            current_key: Arc::new(RwLock::new(None)),
            next_key: Arc::new(RwLock::new(None)),
            previous_keys: Arc::new(DashMap::new()),
            rotation_interval,
        }
    }

    pub fn set_current_key(&self, key_id: u64, key: Vec<u8>, current_time: u64) {
        let expires_at = current_time + self.rotation_interval;
        let key_info = KeyInfo {
            key_id,
            key,
            created_at: current_time,
            expires_at,
        };
        *self.current_key.write() = Some(key_info);
    }

    pub fn set_next_key(&self, key_id: u64, key: Vec<u8>, current_time: u64) {
        let expires_at = current_time + self.rotation_interval;
        let key_info = KeyInfo {
            key_id,
            key,
            created_at: current_time,
            expires_at,
        };
        *self.next_key.write() = Some(key_info);
    }

    pub fn get_current_key(&self) -> Option<KeyInfo> {
        self.current_key.read().clone()
    }

    pub fn rotate_key(&self, current_time: u64) -> bool {
        let next = self.next_key.write().take();
        if let Some(next_key) = next {
            let old = self.current_key.write().replace(next_key.clone());
            if let Some(old_key) = old {
                self.previous_keys.insert(old_key.key_id, old_key);
            }
            true
        } else {
            false
        }
    }

    pub fn get_key_by_id(&self, key_id: u64) -> Option<KeyInfo> {
        if let Some(current) = self.current_key.read().as_ref() {
            if current.key_id == key_id {
                return Some(current.clone());
            }
        }
        
        if let Some(next) = self.next_key.read().as_ref() {
            if next.key_id == key_id {
                return Some(next.clone());
            }
        }
        
        self.previous_keys.get(&key_id).map(|r| r.clone())
    }

    pub fn is_key_valid(&self, key_id: u64, current_time: u64) -> bool {
        if let Some(key) = self.get_key_by_id(key_id) {
            key.expires_at > current_time
        } else {
            false
        }
    }

    pub fn cleanup_expired_keys(&self, current_time: u64) {
        if let Some(current) = self.current_key.read().as_ref() {
            if current.expires_at <= current_time {
                self.rotate_key(current_time);
            }
        }
        
        self.previous_keys.retain(|_, v| v.expires_at > current_time);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_manager_set_and_get() {
        let manager = KeyManager::new(3600);
        manager.set_current_key(1, vec![0u8; 32], 0);
        
        let key = manager.get_current_key();
        assert!(key.is_some());
        assert_eq!(key.unwrap().key_id, 1);
    }

    #[test]
    fn test_key_rotation() {
        let manager = KeyManager::new(3600);
        manager.set_current_key(1, vec![1u8; 32], 0);
        manager.set_next_key(2, vec![2u8; 32], 0);
        
        assert!(manager.rotate_key(3600));
        
        let key = manager.get_current_key();
        assert!(key.is_some());
        assert_eq!(key.unwrap().key_id, 2);
    }

    #[test]
    fn test_key_by_id() {
        let manager = KeyManager::new(3600);
        manager.set_current_key(1, vec![1u8; 32], 0);
        
        let key = manager.get_key_by_id(1);
        assert!(key.is_some());
        assert_eq!(key.unwrap().key, vec![1u8; 32]);
    }
}
