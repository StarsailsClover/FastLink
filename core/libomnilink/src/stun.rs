//! FastLink STUN protocol module
//!
//! RFC 5389 STUN protocol implementation

use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;
use crc32fast::Hasher;
use bytes::{Bytes, BytesMut};
use thiserror::Error;
use tracing::{debug, warn};

/// STUN message type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    BindingRequest,
    BindingResponse,
    BindingError,
    SharedSecretRequest,
    SharedSecretResponse,
    SharedSecretError,
    Other(u16),
}

impl From<u16> for MessageType {
    fn from(value: u16) -> Self {
        match value {
            0x0001 => MessageType::BindingRequest,
            0x0101 => MessageType::BindingResponse,
            0x0111 => MessageType::BindingError,
            0x0002 => MessageType::SharedSecretRequest,
            0x0102 => MessageType::SharedSecretResponse,
            0x0112 => MessageType::SharedSecretError,
            _ => MessageType::Other(value),
        }
    }
}

impl From<MessageType> for u16 {
    fn from(value: MessageType) -> Self {
        match value {
            MessageType::BindingRequest => 0x0001,
            MessageType::BindingResponse => 0x0101,
            MessageType::BindingError => 0x0111,
            MessageType::SharedSecretRequest => 0x0002,
            MessageType::SharedSecretResponse => 0x0102,
            MessageType::SharedSecretError => 0x0112,
            MessageType::Other(v) => v,
        }
    }
}

/// STUN attribute type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributeType {
    MappedAddress,
    ResponseAddress,
    ChangeRequest,
    SourceAddress,
    ChangedAddress,
    Username,
    Password,
    MessageIntegrity,
    ErrorCode,
    UnknownAttributes,
    ReflectedFrom,
    Realm,
    Nonce,
    XorMappedAddress,
    Software,
    AlternateServer,
    Fingerprint,
    Other(u16),
}

impl From<u16> for AttributeType {
    fn from(value: u16) -> Self {
        match value {
            0x0001 => AttributeType::MappedAddress,
            0x0002 => AttributeType::ResponseAddress,
            0x0003 => AttributeType::ChangeRequest,
            0x0004 => AttributeType::SourceAddress,
            0x0005 => AttributeType::ChangedAddress,
            0x0006 => AttributeType::Username,
            0x0007 => AttributeType::Password,
            0x0008 => AttributeType::MessageIntegrity,
            0x0009 => AttributeType::ErrorCode,
            0x000A => AttributeType::UnknownAttributes,
            0x000B => AttributeType::ReflectedFrom,
            0x0014 => AttributeType::Realm,
            0x0015 => AttributeType::Nonce,
            0x0020 => AttributeType::XorMappedAddress,
            0x8022 => AttributeType::Software,
            0x8023 => AttributeType::AlternateServer,
            0x8028 => AttributeType::Fingerprint,
            _ => AttributeType::Other(value),
        }
    }
}

impl From<AttributeType> for u16 {
    fn from(value: AttributeType) -> Self {
        match value {
            AttributeType::MappedAddress => 0x0001,
            AttributeType::ResponseAddress => 0x0002,
            AttributeType::ChangeRequest => 0x0003,
            AttributeType::SourceAddress => 0x0004,
            AttributeType::ChangedAddress => 0x0005,
            AttributeType::Username => 0x0006,
            AttributeType::Password => 0x0007,
            AttributeType::MessageIntegrity => 0x0008,
            AttributeType::ErrorCode => 0x0009,
            AttributeType::UnknownAttributes => 0x000A,
            AttributeType::ReflectedFrom => 0x000B,
            AttributeType::Realm => 0x0014,
            AttributeType::Nonce => 0x0015,
            AttributeType::XorMappedAddress => 0x0020,
            AttributeType::Software => 0x8022,
            AttributeType::AlternateServer => 0x8023,
            AttributeType::Fingerprint => 0x8028,
            AttributeType::Other(v) => v,
        }
    }
}

/// STUN attribute
#[derive(Debug, Clone)]
pub enum StunAttribute {
    MappedAddress(SocketAddr),
    XorMappedAddress(SocketAddr),
    Software(String),
    Username(String),
    MessageIntegrity([u8; 20]),
    ErrorCode(u16, String),
    Fingerprint(u32),
}

/// STUN message
#[derive(Debug, Clone)]
pub struct StunMessage {
    pub message_type: MessageType,
    pub transaction_id: [u8; 12],
    pub attributes: Vec<StunAttribute>,
}

impl StunMessage {
    /// Create a new binding request
    pub fn new_binding_request() -> Self {
        let transaction_id = rand::random();
        Self {
            message_type: MessageType::BindingRequest,
            transaction_id,
            attributes: Vec::new(),
        }
    }
    
    /// Add software attribute
    pub fn add_software(&mut self, software: String) {
        self.attributes.push(StunAttribute::Software(software));
    }
    
    /// Add fingerprint attribute
    pub fn add_fingerprint(&mut self) {
        let fingerprint = self.calculate_fingerprint();
        self.attributes.push(StunAttribute::Fingerprint(fingerprint));
    }
    
    /// Calculate fingerprint
    fn calculate_fingerprint(&self) -> u32 {
        let bytes = self.encode_without_fingerprint();
        let mut hasher = Hasher::new();
        hasher.update(&bytes);
        hasher.finalize() ^ 0x5354554E
    }
    
    /// Encode message without fingerprint
    fn encode_without_fingerprint(&self) -> Vec<u8> {
        let mut result = Vec::new();
        
        let message_type: u16 = self.message_type.into();
        result.extend_from_slice(&message_type.to_be_bytes());
        
        let mut length = 0;
        for attr in &self.attributes {
            if !matches!(attr, StunAttribute::Fingerprint(_)) {
                length += self.attribute_length(attr);
            }
        }
        result.extend_from_slice(&length.to_be_bytes());
        
        result.extend_from_slice(&self.transaction_id);
        
        for attr in &self.attributes {
            if !matches!(attr, StunAttribute::Fingerprint(_)) {
                self.encode_attribute(attr, &mut result);
            }
        }
        
        result
    }
    
    /// Encode the message
    pub fn encode(&self) -> Vec<u8> {
        let mut result = Vec::new();
        
        let message_type: u16 = self.message_type.into();
        result.extend_from_slice(&message_type.to_be_bytes());
        
        let mut length = 0;
        for attr in &self.attributes {
            length += self.attribute_length(attr);
        }
        result.extend_from_slice(&length.to_be_bytes());
        
        result.extend_from_slice(&self.transaction_id);
        
        for attr in &self.attributes {
            self.encode_attribute(attr, &mut result);
        }
        
        result
    }
    
    /// Encode a single attribute
    fn encode_attribute(&self, attr: &StunAttribute, result: &mut Vec<u8>) {
        let attribute_type = AttributeType::from(attr);
        let attribute_type_u16: u16 = attribute_type.into();
        result.extend_from_slice(&attribute_type_u16.to_be_bytes());
        
        let length = self.attribute_value_length(attr);
        result.extend_from_slice(&length.to_be_bytes());
        
        match attr {
            StunAttribute::Software(software) => {
                result.extend_from_slice(software.as_bytes());
                self.pad_to_4_bytes(result, software.len());
            }
            StunAttribute::MappedAddress(addr) => {
                self.encode_address(addr, result);
            }
            StunAttribute::XorMappedAddress(addr) => {
                self.encode_xor_address(addr, result);
            }
            StunAttribute::Username(username) => {
                result.extend_from_slice(username.as_bytes());
                self.pad_to_4_bytes(result, username.len());
            }
            StunAttribute::Fingerprint(fp) => {
                result.extend_from_slice(&fp.to_be_bytes());
            }
            _ => {}
        }
    }
    
    /// Encode an address attribute
    fn encode_address(&self, addr: &SocketAddr, result: &mut Vec<u8>) {
        result.push(0x00);
        match addr {
            SocketAddr::V4(_) => {
                result.push(0x01);
            }
            SocketAddr::V6(_) => {
                result.push(0x02);
            }
        }
        result.extend_from_slice(&addr.port().to_be_bytes());
        
        match addr {
            SocketAddr::V4(v4) => {
                result.extend_from_slice(&v4.ip().octets());
            }
            SocketAddr::V6(v6) => {
                result.extend_from_slice(&v6.ip().octets());
            }
        }
    }
    
    /// Encode an XOR-mapped address attribute
    fn encode_xor_address(&self, addr: &SocketAddr, result: &mut Vec<u8>) {
        result.push(0x00);
        match addr {
            SocketAddr::V4(_) => {
                result.push(0x01);
            }
            SocketAddr::V6(_) => {
                result.push(0x02);
            }
        }
        
        let xor_port = addr.port() ^ ((self.transaction_id[0] as u16) << 8 | self.transaction_id[1] as u16);
        result.extend_from_slice(&xor_port.to_be_bytes());
        
        match addr {
            SocketAddr::V4(v4) => {
                let mut octets = v4.ip().octets();
                octets[0] ^= self.transaction_id[0];
                octets[1] ^= self.transaction_id[1];
                octets[2] ^= self.transaction_id[2];
                octets[3] ^= self.transaction_id[3];
                result.extend_from_slice(&octets);
            }
            SocketAddr::V6(v6) => {
                let mut octets = v6.ip().octets();
                for i in 0..16 {
                    octets[i] ^= self.transaction_id[i.min(11)];
                }
                result.extend_from_slice(&octets);
            }
        }
    }
    
    /// Pad to 4-byte boundary
    fn pad_to_4_bytes(&self, result: &mut Vec<u8>, current_length: usize) {
        let padding = (4 - (current_length % 4)) % 4;
        for _ in 0..padding {
            result.push(0x00);
        }
    }
    
    /// Get attribute value length
    fn attribute_value_length(&self, attr: &StunAttribute) -> u16 {
        match attr {
            StunAttribute::Software(software) => software.len() as u16,
            StunAttribute::MappedAddress(addr) => match addr {
                SocketAddr::V4(_) => 8,
                SocketAddr::V6(_) => 20,
            },
            StunAttribute::XorMappedAddress(addr) => match addr {
                SocketAddr::V4(_) => 8,
                SocketAddr::V6(_) => 20,
            },
            StunAttribute::Username(username) => username.len() as u16,
            StunAttribute::Fingerprint(_) => 4,
            _ => 0,
        }
    }
    
    /// Get total attribute length including header
    fn attribute_length(&self, attr: &StunAttribute) -> u16 {
        4 + self.attribute_value_length(attr)
    }
    
    /// Decode a STUN message from bytes
    pub fn decode(bytes: &[u8]) -> Result<Self, StunError> {
        if bytes.len() < 20 {
            return Err(StunError::InvalidPacket);
        }
        
        let message_type = MessageType::from(u16::from_be_bytes([bytes[0], bytes[1]]));
        let length = u16::from_be_bytes([bytes[2], bytes[3]]) as usize;
        
        let mut transaction_id = [0u8; 12];
        transaction_id.copy_from_slice(&bytes[4..16]);
        
        let mut attributes = Vec::new();
        let mut offset = 20;
        
        while offset < 20 + length {
            if offset + 4 > bytes.len() {
                break;
            }
            
            let attr_type = u16::from_be_bytes([bytes[offset], bytes[offset + 1]]);
            let attr_length = u16::from_be_bytes([bytes[offset + 2], bytes[offset + 3]]) as usize;
            
            offset += 4;
            
            if offset + attr_length > bytes.len() {
                break;
            }
            
            let attr_value = &bytes[offset..offset + attr_length];
            
            match AttributeType::from(attr_type) {
                AttributeType::XorMappedAddress => {
                    if let Some(addr) = Self::decode_xor_address(attr_value, transaction_id) {
                        attributes.push(StunAttribute::XorMappedAddress(addr));
                    }
                }
                AttributeType::MappedAddress => {
                    if let Some(addr) = Self::decode_address(attr_value) {
                        attributes.push(StunAttribute::MappedAddress(addr));
                    }
                }
                AttributeType::Software => {
                    if let Ok(s) = String::from_utf8(attr_value.to_vec()) {
                        attributes.push(StunAttribute::Software(s));
                    }
                }
                _ => {}
            }
            
            offset += attr_length + (4 - attr_length % 4) % 4;
        }
        
        Ok(Self {
            message_type,
            transaction_id,
            attributes,
        })
    }
    
    /// Decode a standard address
    fn decode_address(bytes: &[u8]) -> Option<SocketAddr> {
        if bytes.len() < 8 {
            return None;
        }
        
        let family = bytes[1];
        let port = u16::from_be_bytes([bytes[2], bytes[3]]);
        
        match family {
            0x01 => {
                let ip = std::net::Ipv4Addr::new(bytes[4], bytes[5], bytes[6], bytes[7]);
                Some(SocketAddr::new(std::net::IpAddr::V4(ip), port))
            }
            0x02 => {
                if bytes.len() < 20 {
                    return None;
                }
                let ip = std::net::Ipv6Addr::new(
                    u16::from_be_bytes([bytes[4], bytes[5]]),
                    u16::from_be_bytes([bytes[6], bytes[7]]),
                    u16::from_be_bytes([bytes[8], bytes[9]]),
                    u16::from_be_bytes([bytes[10], bytes[11]]),
                    u16::from_be_bytes([bytes[12], bytes[13]]),
                    u16::from_be_bytes([bytes[14], bytes[15]]),
                    u16::from_be_bytes([bytes[16], bytes[17]]),
                    u16::from_be_bytes([bytes[18], bytes[19]]),
                );
                Some(SocketAddr::new(std::net::IpAddr::V6(ip), port))
            }
            _ => None,
        }
    }
    
    /// Decode an XOR-mapped address
    fn decode_xor_address(bytes: &[u8], transaction_id: [u8; 12]) -> Option<SocketAddr> {
        if bytes.len() < 8 {
            return None;
        }
        
        let family = bytes[1];
        let xor_port = u16::from_be_bytes([bytes[2], bytes[3]]);
        let port = xor_port ^ ((transaction_id[0] as u16) << 8 | transaction_id[1] as u16);
        
        match family {
            0x01 => {
                let mut octets = [bytes[4], bytes[5], bytes[6], bytes[7]];
                octets[0] ^= transaction_id[0];
                octets[1] ^= transaction_id[1];
                octets[2] ^= transaction_id[2];
                octets[3] ^= transaction_id[3];
                let ip = std::net::Ipv4Addr::new(octets[0], octets[1], octets[2], octets[3]);
                Some(SocketAddr::new(std::net::IpAddr::V4(ip), port))
            }
            0x02 => {
                if bytes.len() < 20 {
                    return None;
                }
                let mut octets = [0u8; 16];
                for i in 0..16 {
                    octets[i] = bytes[4 + i] ^ transaction_id[i.min(11)];
                }
                let ip = std::net::Ipv6Addr::new(
                    u16::from_be_bytes([octets[0], octets[1]]),
                    u16::from_be_bytes([octets[2], octets[3]]),
                    u16::from_be_bytes([octets[4], octets[5]]),
                    u16::from_be_bytes([octets[6], octets[7]]),
                    u16::from_be_bytes([octets[8], octets[9]]),
                    u16::from_be_bytes([octets[10], octets[11]]),
                    u16::from_be_bytes([octets[12], octets[13]]),
                    u16::from_be_bytes([octets[14], octets[15]]),
                );
                Some(SocketAddr::new(std::net::IpAddr::V6(ip), port))
            }
            _ => None,
        }
    }
    
    /// Get the XOR-mapped address from the message
    pub fn get_xor_mapped_address(&self) -> Option<SocketAddr> {
        for attr in &self.attributes {
            if let StunAttribute::XorMappedAddress(addr) = attr {
                return Some(*addr);
            }
        }
        None
    }
}

impl From<&StunAttribute> for AttributeType {
    fn from(value: &StunAttribute) -> Self {
        match value {
            StunAttribute::MappedAddress(_) => AttributeType::MappedAddress,
            StunAttribute::XorMappedAddress(_) => AttributeType::XorMappedAddress,
            StunAttribute::Software(_) => AttributeType::Software,
            StunAttribute::Username(_) => AttributeType::Username,
            StunAttribute::MessageIntegrity(_) => AttributeType::MessageIntegrity,
            StunAttribute::ErrorCode(_, _) => AttributeType::ErrorCode,
            StunAttribute::Fingerprint(_) => AttributeType::Fingerprint,
        }
    }
}

/// STUN client for NAT traversal
pub struct StunClient {
    socket: UdpSocket,
    stun_server: SocketAddr,
}

impl StunClient {
    /// Create a new STUN client
    pub fn new(local_addr: SocketAddr, stun_server: SocketAddr) -> Result<Self, StunError> {
        let socket = UdpSocket::bind(local_addr)?;
        socket.set_read_timeout(Some(Duration::from_secs(5)))?;
        Ok(Self { socket, stun_server })
    }
    
    /// Query the STUN server for our public address
    pub fn query_public_address(&self) -> Result<SocketAddr, StunError> {
        let mut request = StunMessage::new_binding_request();
        request.add_software("FastLink/1.0".to_string());
        
        let bytes = request.encode();
        self.socket.send_to(&bytes, self.stun_server)?;
        
        let mut buf = [0u8; 4096];
        let (size, _) = self.socket.recv_from(&mut buf)?;
        
        let response = StunMessage::decode(&buf[..size])?;
        
        if response.message_type != MessageType::BindingResponse {
            return Err(StunError::InvalidResponse);
        }
        
        response.get_xor_mapped_address().ok_or(StunError::NoMappedAddress)
    }
}

#[derive(Debug, Error)]
pub enum StunError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid STUN packet")]
    InvalidPacket,
    #[error("Invalid STUN response")]
    InvalidResponse,
    #[error("No mapped address in response")]
    NoMappedAddress,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stun_message_creation() {
        let request = StunMessage::new_binding_request();
        assert_eq!(request.message_type, MessageType::BindingRequest);
        assert_eq!(request.transaction_id.len(), 12);
    }

    #[test]
    fn test_stun_message_encode_decode() {
        let mut request = StunMessage::new_binding_request();
        request.add_software("Test".to_string());
        
        let encoded = request.encode();
        let decoded = StunMessage::decode(&encoded).unwrap();
        
        assert_eq!(request.message_type, decoded.message_type);
        assert_eq!(request.transaction_id, decoded.transaction_id);
    }
}
