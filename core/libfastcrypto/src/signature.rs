//! FastLink Signature Module
//!
//! Digital signature implementation using Ed25519

use ring::signature::{self, KeyPair as RingKeyPair, ED25519};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SignatureError {
    #[error("Invalid private key length")]
    InvalidKeyLength,
    #[error("Failed to create key pair from PKCS8")]
    InvalidPkcs8,
    #[error("Failed to recreate key pair")]
    InvalidKeyPair,
    #[error("Failed to generate key pair")]
    GenerationFailed,
}

#[derive(Debug, Clone)]
pub struct Ed25519KeyPair {
    public_key: Vec<u8>,
    private_key: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature(pub Vec<u8>);

impl Ed25519KeyPair {
    pub fn generate() -> Self {
        let rng = ring::rand::SystemRandom::new();
        let pkcs8_bytes = signature::Ed25519KeyPair::generate_pkcs8(&rng)
            .expect("Failed to generate key pair");
        
        let key_pair = signature::Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref())
            .expect("Failed to create key pair from PKCS8");
        
        let public_key = key_pair.public_key().as_ref().to_vec();
        let private_key = pkcs8_bytes.as_ref().to_vec();
        
        Self { public_key, private_key }
    }
    
    pub fn from_private_key(private_key: &[u8]) -> Result<Self, SignatureError> {
        if private_key.len() < 32 {
            return Err(SignatureError::InvalidKeyLength);
        }
        
        let key_pair = signature::Ed25519KeyPair::from_pkcs8(private_key)
            .map_err(|_| SignatureError::InvalidPkcs8)?;
        
        let public_key = key_pair.public_key().as_ref().to_vec();
        let private_key = private_key.to_vec();
        
        Ok(Self { public_key, private_key })
    }
    
    pub fn public_key(&self) -> &[u8] {
        &self.public_key
    }
    
    pub fn private_key(&self) -> &[u8] {
        &self.private_key
    }
    
    pub fn sign(&self, message: &[u8]) -> Signature {
        let key_pair = signature::Ed25519KeyPair::from_pkcs8(&self.private_key)
            .expect("Failed to recreate key pair");
        
        let signature = key_pair.sign(message);
        Signature(signature.as_ref().to_vec())
    }
}

pub type KeyPair = Ed25519KeyPair;

pub fn verify(public_key: &[u8], message: &[u8], signature: &[u8]) -> bool {
    let public_key = signature::UnparsedPublicKey::new(
        &signature::ED25519,
        public_key,
    );
    
    public_key.verify(message, signature).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_generation() {
        let key_pair = KeyPair::generate();
        assert_eq!(key_pair.public_key().len(), 32);
        assert!(key_pair.private_key().len() >= 32);
    }

    #[test]
    fn test_sign_and_verify() {
        let key_pair = KeyPair::generate();
        let message = b"Hello, FastLink!";
        
        let signature = key_pair.sign(message);
        assert!(verify(key_pair.public_key(), message, &signature.0));
    }

    #[test]
    fn test_invalid_signature() {
        let key_pair = KeyPair::generate();
        let message = b"Hello, FastLink!";
        
        let signature = key_pair.sign(message);
        assert!(!verify(key_pair.public_key(), b"Wrong message", &signature.0));
    }

    #[test]
    fn test_wrong_public_key() {
        let key_pair1 = KeyPair::generate();
        let key_pair2 = KeyPair::generate();
        let message = b"Hello, FastLink!";
        
        let signature = key_pair1.sign(message);
        assert!(!verify(key_pair2.public_key(), message, &signature.0));
    }
}
