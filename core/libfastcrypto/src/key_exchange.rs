//! FastLink Key Exchange Module
//!
//! Key exchange implementation using X25519

use x25519_dalek::{PublicKey, StaticSecret};
use serde::{Deserialize, Serialize};
use rand::rngs::OsRng;
use rand::SeedableRng;
use rand_chacha::ChaChaRng;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyExchangePublicKey(pub [u8; 32]);

#[derive(Clone)]
pub struct KeyExchangePrivateKey {
    secret: StaticSecret,
}

impl std::fmt::Debug for KeyExchangePrivateKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "KeyExchangePrivateKey {{ ... }}")
    }
}

impl KeyExchangePrivateKey {
    pub fn generate() -> Self {
        let secret = StaticSecret::random_from_rng(OsRng);
        Self { secret }
    }
    
    pub fn from_bytes(bytes: &[u8; 32]) -> Self {
        let mut rng = ChaChaRng::from_seed(*bytes);
        let secret = StaticSecret::random_from_rng(&mut rng);
        Self { secret }
    }
    
    pub fn public_key(&self) -> KeyExchangePublicKey {
        let public = PublicKey::from(&self.secret);
        let mut pk = [0u8; 32];
        pk.copy_from_slice(public.as_bytes());
        KeyExchangePublicKey(pk)
    }
    
    pub fn diffie_hellman(&self, peer_public: &KeyExchangePublicKey) -> SharedSecret {
        let peer_key = PublicKey::from(peer_public.0);
        let shared = self.secret.diffie_hellman(&peer_key);
        SharedSecret(shared.as_bytes().to_vec())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedSecret(pub Vec<u8>);

impl SharedSecret {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

pub fn generate_keypair() -> (KeyExchangePrivateKey, KeyExchangePublicKey) {
    let private = KeyExchangePrivateKey::generate();
    let public = private.public_key();
    (private, public)
}

pub fn derive_shared_secret(
    private_key: &[u8; 32],
    peer_public_key: &[u8; 32],
) -> SharedSecret {
    let private = KeyExchangePrivateKey::from_bytes(private_key);
    let peer = KeyExchangePublicKey(*peer_public_key);
    private.diffie_hellman(&peer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_generation() {
        let (private, public) = generate_keypair();
        assert_eq!(public.0.len(), 32);
        let _ = private.public_key();
    }

    #[test]
    fn test_key_exchange() {
        let (alice_private, alice_public) = generate_keypair();
        let (bob_private, bob_public) = generate_keypair();
        
        let alice_shared = alice_private.diffie_hellman(&bob_public);
        let bob_shared = bob_private.diffie_hellman(&alice_public);
        
        assert_eq!(alice_shared.0, bob_shared.0);
    }

    #[test]
    fn test_derive_shared_secret() {
        let (alice_private, _) = generate_keypair();
        let (_, bob_public) = generate_keypair();
        
        let shared = derive_shared_secret(
            alice_private.public_key().0.as_ref().try_into().unwrap(),
            &bob_public.0,
        );
        
        assert_eq!(shared.0.len(), 32);
    }
}
