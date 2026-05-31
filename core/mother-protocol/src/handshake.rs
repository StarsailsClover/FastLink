//! FastLink Handshake Module
//!
//! Protocol handshake and negotiation

use libfastcrypto::{KeyPair, Signature};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HandshakeError {
    #[error("Invalid version")]
    InvalidVersion,
    #[error("Signature verification failed")]
    InvalidSignature,
    #[error("Timeout")]
    Timeout,
    #[error("Protocol negotiation failed")]
    NegotiationFailed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeMessage {
    pub version: u16,
    pub protocol_id: u16,
    pub public_key: Vec<u8>,
    pub signature: Vec<u8>,
    pub timestamp: u64,
    pub nonce: Vec<u8>,
}

impl HandshakeMessage {
    pub fn new(protocol_id: u16, key_pair: &KeyPair, nonce: Vec<u8>) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let data = Self::create_sign_data(protocol_id, timestamp, &nonce);
        let signature = key_pair.sign(&data);
        
        Self {
            version: 1,
            protocol_id,
            public_key: key_pair.public_key().to_vec(),
            signature: signature.0,
            timestamp,
            nonce,
        }
    }
    
    pub fn create_sign_data(protocol_id: u16, timestamp: u64, nonce: &[u8]) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(&protocol_id.to_be_bytes());
        data.extend_from_slice(&timestamp.to_be_bytes());
        data.extend_from_slice(nonce);
        data
    }
    
    pub fn verify(&self, key_pair: &KeyPair) -> Result<(), HandshakeError> {
        if self.version != 1 {
            return Err(HandshakeError::InvalidVersion);
        }
        
        let data = Self::create_sign_data(self.protocol_id, self.timestamp, &self.nonce);
        
        if libfastcrypto::verify(key_pair.public_key(), &data, &self.signature) {
            Ok(())
        } else {
            Err(HandshakeError::InvalidSignature)
        }
    }
}

pub struct HandshakeState {
    pub is_initiator: bool,
    pub remote_public_key: Option<Vec<u8>>,
    pub shared_secret: Option<Vec<u8>>,
}

impl HandshakeState {
    pub fn new(is_initiator: bool) -> Self {
        Self {
            is_initiator,
            remote_public_key: None,
            shared_secret: None,
        }
    }
    
    pub fn set_remote_public_key(&mut self, public_key: Vec<u8>) {
        self.remote_public_key = Some(public_key);
    }
    
    pub fn set_shared_secret(&mut self, secret: Vec<u8>) {
        self.shared_secret = Some(secret);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handshake_message_creation() {
        let key_pair = KeyPair::generate();
        let nonce = vec![0u8; 32];
        
        let message = HandshakeMessage::new(1, &key_pair, nonce.clone());
        
        assert_eq!(message.version, 1);
        assert_eq!(message.protocol_id, 1);
        assert!(message.verify(&key_pair).is_ok());
    }

    #[test]
    fn test_handshake_message_verification() {
        let key_pair1 = KeyPair::generate();
        let key_pair2 = KeyPair::generate();
        let nonce = vec![0u8; 32];
        
        let message = HandshakeMessage::new(1, &key_pair1, nonce);
        
        assert!(message.verify(&key_pair1).is_ok());
        assert!(message.verify(&key_pair2).is_err());
    }
}
