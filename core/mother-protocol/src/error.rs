//! FastLink Error Types
//!
//! Unified error codes following the protocol specification

use thiserror::Error;
use serde::{Deserialize, Serialize};

pub const ERR_OK: u32 = 0x00000000;
pub const ERR_VERSION: u32 = 0x00000001;
pub const ERR_HANDSHAKE: u32 = 0x00000002;
pub const ERR_AUTH: u32 = 0x00000003;
pub const ERR_DECRYPT: u32 = 0x00000004;
pub const ERR_CHECKSUM: u32 = 0x00000005;
pub const ERR_SEQUENCE: u32 = 0x00000006;
pub const ERR_TIMEOUT: u32 = 0x00000007;
pub const ERR_BUSY: u32 = 0x00000008;
pub const ERR_LIMIT: u32 = 0x00000009;
pub const ERR_FORMAT: u32 = 0x0000000A;
pub const ERR_CRYPTO: u32 = 0x0000000B;
pub const ERR_INTERNAL: u32 = 0x0000000C;
pub const ERR_CLOSED: u32 = 0x0000000D;
pub const ERR_OVERFLOW: u32 = 0x0000000E;
pub const ERR_MTU: u32 = 0x0000000F;

pub const ERR_PARAM: u32 = 0x00010000;
pub const ERR_NETWORK: u32 = 0x00020000;
pub const ERR_CRYPTO_BASE: u32 = 0x00030000;
pub const ERR_DHT: u32 = 0x00040000;
pub const ERR_NAT: u32 = 0x00050000;

#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum Error {
    #[error("Success")]
    Ok,
    
    #[error("Version mismatch")]
    VersionMismatch,
    
    #[error("Handshake failed")]
    HandshakeFailed,
    
    #[error("Authentication failed")]
    AuthFailed,
    
    #[error("Decryption failed")]
    DecryptFailed,
    
    #[error("Checksum mismatch")]
    ChecksumMismatch,
    
    #[error("Sequence error")]
    SequenceError,
    
    #[error("Operation timeout")]
    Timeout,
    
    #[error("Server busy")]
    Busy,
    
    #[error("Rate limit exceeded")]
    RateLimit,
    
    #[error("Invalid format")]
    InvalidFormat,
    
    #[error("Cryptography error")]
    CryptoError,
    
    #[error("Internal error")]
    Internal,
    
    #[error("Connection closed")]
    ConnectionClosed,
    
    #[error("Buffer overflow")]
    BufferOverflow,
    
    #[error("MTU exceeded")]
    MtuExceeded,
    
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("NAT traversal failed: {0}")]
    NatTraversalFailed(String),
    
    #[error("DHT error: {0}")]
    DhtError(String),
    
    #[error("IO error: {0}")]
    IoError(String),
}

impl Error {
    pub fn error_code(&self) -> u32 {
        match self {
            Error::Ok => ERR_OK,
            Error::VersionMismatch => ERR_VERSION,
            Error::HandshakeFailed => ERR_HANDSHAKE,
            Error::AuthFailed => ERR_AUTH,
            Error::DecryptFailed => ERR_DECRYPT,
            Error::ChecksumMismatch => ERR_CHECKSUM,
            Error::SequenceError => ERR_SEQUENCE,
            Error::Timeout => ERR_TIMEOUT,
            Error::Busy => ERR_BUSY,
            Error::RateLimit => ERR_LIMIT,
            Error::InvalidFormat => ERR_FORMAT,
            Error::CryptoError => ERR_CRYPTO,
            Error::Internal => ERR_INTERNAL,
            Error::ConnectionClosed => ERR_CLOSED,
            Error::BufferOverflow => ERR_OVERFLOW,
            Error::MtuExceeded => ERR_MTU,
            Error::InvalidParameter(_) => ERR_PARAM,
            Error::NetworkError(_) => ERR_NETWORK,
            Error::NatTraversalFailed(_) => ERR_NAT,
            Error::DhtError(_) => ERR_DHT,
            Error::IoError(_) => ERR_NETWORK,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IoError(err.to_string())
    }
}

impl From<ring::error::Unspecified> for Error {
    fn from(_: ring::error::Unspecified) -> Self {
        Error::CryptoError
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        assert_eq!(Error::Ok.error_code(), ERR_OK);
        assert_eq!(Error::VersionMismatch.error_code(), ERR_VERSION);
        assert_eq!(Error::HandshakeFailed.error_code(), ERR_HANDSHAKE);
        assert_eq!(Error::Timeout.error_code(), ERR_TIMEOUT);
    }

    #[test]
    fn test_error_display() {
        let err = Error::Timeout;
        assert_eq!(err.to_string(), "Operation timeout");
    }
}
