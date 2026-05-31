//! FastLink Symmetric Encryption Module
//!
//! Symmetric encryption using AES-256-GCM

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use ring::rand::SecureRandom;
use serde::{Deserialize, Serialize};

const NONCE_SIZE: usize = 12;
const KEY_SIZE: usize = 32;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    pub nonce: Vec<u8>,
    pub ciphertext: Vec<u8>,
    pub tag: Vec<u8>,
}

pub struct Aes256GcmCipher {
    cipher: Aes256Gcm,
}

impl Aes256GcmCipher {
    pub fn new(key: &[u8; KEY_SIZE]) -> Self {
        let cipher = Aes256Gcm::new(key.into());
        Self { cipher }
    }
    
    pub fn generate_key() -> [u8; KEY_SIZE] {
        let mut key = [0u8; KEY_SIZE];
        let rng = ring::rand::SystemRandom::new();
        rng.fill(&mut key).unwrap();
        key
    }
    
    pub fn encrypt(&self, plaintext: &[u8]) -> EncryptedData {
        let rng = ring::rand::SystemRandom::new();
        let mut nonce_bytes = [0u8; NONCE_SIZE];
        rng.fill(&mut nonce_bytes).unwrap();
        
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = self.cipher
            .encrypt(nonce, plaintext)
            .expect("Encryption failed");
        
        EncryptedData {
            nonce: nonce_bytes.to_vec(),
            ciphertext,
            tag: Vec::new(),
        }
    }
    
    pub fn decrypt(&self, encrypted: &EncryptedData) -> Result<Vec<u8>, &'static str> {
        if encrypted.nonce.len() != NONCE_SIZE {
            return Err("Invalid nonce length");
        }
        
        let nonce = Nonce::from_slice(&encrypted.nonce);
        self.cipher
            .decrypt(nonce, encrypted.ciphertext.as_ref())
            .map_err(|_| "Decryption failed")
    }
}

pub type ChaCha20Poly1305Cipher = chacha20poly1305::ChaCha20Poly1305;

pub struct ChaCha20Poly1305 {
    cipher: ChaCha20Poly1305Cipher,
}

impl ChaCha20Poly1305 {
    pub fn new(key: &[u8; 32]) -> Self {
        let cipher = ChaCha20Poly1305Cipher::new(key.into());
        Self { cipher }
    }
    
    pub fn encrypt(&self, nonce: &[u8], plaintext: &[u8]) -> Vec<u8> {
        let nonce = chacha20poly1305::Nonce::from_slice(nonce);
        self.cipher
            .encrypt(nonce, plaintext)
            .expect("Encryption failed")
    }
    
    pub fn decrypt(&self, nonce: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>, &'static str> {
        let nonce = chacha20poly1305::Nonce::from_slice(nonce);
        self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| "Decryption failed")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes256gcm_key_generation() {
        let key = Aes256GcmCipher::generate_key();
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn test_aes256gcm_encrypt_decrypt() {
        let key = Aes256GcmCipher::generate_key();
        let cipher = Aes256GcmCipher::new(&key);
        
        let plaintext = b"Hello, FastLink!";
        let encrypted = cipher.encrypt(plaintext);
        
        assert!(!encrypted.ciphertext.is_empty());
        assert_eq!(encrypted.nonce.len(), 12);
        
        let decrypted = cipher.decrypt(&encrypted).unwrap();
        assert_eq!(&decrypted[..], plaintext);
    }

    #[test]
    fn test_chacha20poly1305() {
        let key = [0u8; 32];
        let cipher = ChaCha20Poly1305::new(&key);
        
        let plaintext = b"Hello, FastLink!";
        let nonce = [0u8; 12];
        
        let ciphertext = cipher.encrypt(&nonce, plaintext);
        let decrypted = cipher.decrypt(&nonce, &ciphertext).unwrap();
        
        assert_eq!(&decrypted[..], plaintext);
    }
}
