//! FastLink Hash Functions Module
//!
//! Hash functions using BLAKE3 and SHA-256

use blake3::Hasher;
use sha2::{Sha256, Digest};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hash(pub Vec<u8>);

pub fn blake3_hash(data: &[u8]) -> Hash {
    let mut hasher = Hasher::new();
    hasher.update(data);
    let hash = hasher.finalize();
    Hash(hash.as_bytes().to_vec())
}

pub fn blake3_verify(data: &[u8], hash: &[u8]) -> bool {
    let computed = blake3_hash(data);
    computed.0 == hash
}

pub fn blake3_keyed_hash(data: &[u8], key: &[u8]) -> Hash {
    let mut key_array = [0u8; 32];
    key_array[..key.len().min(32)].copy_from_slice(&key[..key.len().min(32)]);
    let mut hasher = Hasher::new_keyed(&key_array);
    hasher.update(data);
    let hash = hasher.finalize();
    Hash(hash.as_bytes().to_vec())
}

pub fn blake3_derive_key(data: &[u8], context: &str) -> Hash {
    let hash = blake3::derive_key(context, data);
    Hash(hash.to_vec())
}

pub fn sha256_hash(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

pub fn sha256_verify(data: &[u8], hash: &[u8]) -> bool {
    &sha256_hash(data) == hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blake3_hash() {
        let data = b"Hello, FastLink!";
        let hash = blake3_hash(data);
        
        assert_eq!(hash.0.len(), 32);
    }

    #[test]
    fn test_blake3_verify() {
        let data = b"Hello, FastLink!";
        let hash = blake3_hash(data);
        
        assert!(blake3_verify(data, &hash.0));
        assert!(!blake3_verify(b"Wrong data", &hash.0));
    }

    #[test]
    fn test_blake3_keyed_hash() {
        let data = b"Hello, FastLink!";
        let key = b"test_key";
        
        let hash1 = blake3_keyed_hash(data, key);
        let hash2 = blake3_keyed_hash(data, key);
        
        assert_eq!(hash1.0, hash2.0);
        
        let hash3 = blake3_keyed_hash(data, b"different_key");
        assert_ne!(hash1.0, hash3.0);
    }

    #[test]
    fn test_sha256_hash() {
        let data = b"Hello, FastLink!";
        let hash = sha256_hash(data);
        
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_sha256_verify() {
        let data = b"Hello, FastLink!";
        let hash = sha256_hash(data);
        
        assert!(sha256_verify(data, &hash));
        assert!(!sha256_verify(b"Wrong data", &hash));
    }
}
