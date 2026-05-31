//! FastLink Key Derivation Functions Module
//!
//! KDF implementations using HKDF and BLAKE3

use blake3::derive_key;
use hkdf::Hkdf;
use serde::{Deserialize, Serialize};
use sha2::Sha256;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DerivedKey(pub Vec<u8>);

pub fn hkdf_sha256(ikm: &[u8], salt: &[u8], info: &[u8], len: usize) -> DerivedKey {
    let hk = Hkdf::<Sha256>::new(Some(salt), ikm);
    let mut okm = vec![0u8; len];
    hk.expand(info, &mut okm).expect("HKDF expand failed");
    DerivedKey(okm)
}

pub fn blake3_derive(ikm: &[u8], context: &str) -> DerivedKey {
    let key = derive_key(context, ikm);
    DerivedKey(key.to_vec())
}

pub fn blake3_derive_with_salt(ikm: &[u8], context: &str, salt: &[u8]) -> DerivedKey {
    let mut combined_input = Vec::with_capacity(ikm.len() + salt.len());
    combined_input.extend_from_slice(salt);
    combined_input.extend_from_slice(ikm);
    blake3_derive(&combined_input, context)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hkdf_sha256() {
        let ikm = b"input key material";
        let salt = b"salt";
        let info = b"info";
        let derived = hkdf_sha256(ikm, salt, info, 32);
        
        assert_eq!(derived.0.len(), 32);
    }

    #[test]
    fn test_blake3_derive() {
        let ikm = b"input key material";
        let derived = blake3_derive(ikm, "fastlink-key-derivation");
        
        assert_eq!(derived.0.len(), 32);
    }

    #[test]
    fn test_blake3_derive_with_salt() {
        let ikm = b"input key material";
        let salt = b"salt";
        let derived = blake3_derive_with_salt(ikm, "fastlink-key-derivation", salt);
        
        assert_eq!(derived.0.len(), 32);
    }
}
