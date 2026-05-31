//! FastLink Common Error Types

use thiserror::Error;

/// Common error types used across FastLink
#[derive(Error, Debug)]
pub enum CommonError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Crypto error: {0}")]
    Crypto(String),
    
    #[error("Timeout")]
    Timeout,
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Result alias for convenience
pub type Result<T> = std::result::Result<T, CommonError>;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_display() {
        let err = CommonError::Io(std::io::Error::new(std::io::ErrorKind::Other, "test"));
        assert!(err.to_string().contains("test"));
    }
}
