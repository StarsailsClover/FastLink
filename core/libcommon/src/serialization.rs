//! FastLink Serialization Utilities

use std::io::{Read, Write};
use thiserror::Error;

/// Serialization errors
#[derive(Error, Debug)]
pub enum SerializationError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Bincode error: {0}")]
    Bincode(#[from] bincode::Error),
    
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("Invalid data")]
    InvalidData,
}

pub type Result<T> = std::result::Result<T, SerializationError>;

/// Serialize to bytes using bincode
pub fn serialize<T: serde::Serialize>(value: &T) -> Result<Vec<u8>> {
    bincode::serialize(value).map_err(|e| SerializationError::Bincode(e))
}

/// Deserialize from bytes using bincode
pub fn deserialize<T: serde::de::DeserializeOwned>(data: &[u8]) -> Result<T> {
    bincode::deserialize(data).map_err(|e| SerializationError::Bincode(e))
}

/// Serialize to JSON string
pub fn to_json<T: serde::Serialize>(value: &T) -> Result<String> {
    serde_json::to_string(value).map_err(|e| SerializationError::Json(e))
}

/// Deserialize from JSON string
pub fn from_json<T: serde::de::DeserializeOwned>(data: &str) -> Result<T> {
    serde_json::from_str(data).map_err(|e| SerializationError::Json(e))
}

/// Serialize size estimation
pub fn serialized_size<T: serde::Serialize>(value: &T) -> Result<usize> {
    bincode::serialized_size(value)
        .map(|s| s as usize)
        .map_err(|e| SerializationError::Bincode(e))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct TestStruct {
        name: String,
        value: u32,
    }
    
    #[test]
    fn test_serialize_deserialize() {
        let original = TestStruct {
            name: "test".to_string(),
            value: 42,
        };
        
        let bytes = serialize(&original).unwrap();
        let decoded: TestStruct = deserialize(&bytes).unwrap();
        
        assert_eq!(original, decoded);
    }
    
    #[test]
    fn test_json_roundtrip() {
        let original = TestStruct {
            name: "json_test".to_string(),
            value: 123,
        };
        
        let json = to_json(&original).unwrap();
        let decoded: TestStruct = from_json(&json).unwrap();
        
        assert_eq!(original, decoded);
    }
}
