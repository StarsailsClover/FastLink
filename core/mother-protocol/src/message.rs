//! FastLink Mother Protocol Message Format
//!
//! Unified message header definition (28 bytes)

use serde::{Deserialize, Serialize};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    Handshake = 0x00,
    Data = 0x01,
    Heartbeat = 0x02,
    Error = 0x03,
    Control = 0x04,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProtocolType {
    Mother = 0x00,
    P2P = 0x01,
    Server = 0x02,
    Swift = 0x03,
    Games = 0x04,
    Aztec = 0x06,
    Chat = 0x07,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MessageHeader {
    pub magic: u32,
    pub version: u8,
    pub msg_type: u8,
    pub protocol_type: u8,
    pub flags: u8,
    pub session_id: u64,
    pub sequence: u32,
    pub timestamp: u64,
    pub length: u16,
    pub checksum: u32,
}

impl MessageHeader {
    pub const MAGIC: u32 = 0x464C4B48; // "FLKH"
    pub const HEADER_SIZE: usize = 28;
    
    pub fn new(
        msg_type: MessageType,
        protocol_type: ProtocolType,
        session_id: u64,
        sequence: u32,
        length: u16,
    ) -> Self {
        Self {
            magic: Self::MAGIC,
            version: 0x01,
            msg_type: msg_type as u8,
            protocol_type: protocol_type as u8,
            flags: 0,
            session_id,
            sequence,
            timestamp: current_timestamp(),
            length,
            checksum: 0,
        }
    }
    
    pub fn validate(&self) -> bool {
        self.magic == Self::MAGIC && self.version == 0x01
    }
    
    pub fn set_checksum(&mut self, payload: &[u8]) {
        let header_bytes = unsafe {
            std::slice::from_raw_parts(self as *const _ as *const u8, Self::HEADER_SIZE)
        };
        let mut data = Vec::with_capacity(header_bytes.len() + payload.len());
        data.extend_from_slice(header_bytes);
        data.extend_from_slice(payload);
        self.checksum = crc32fast::hash(&data);
    }
    
    pub fn verify_checksum(&self, payload: &[u8]) -> bool {
        let header_bytes = unsafe {
            std::slice::from_raw_parts(self as *const _ as *const u8, Self::HEADER_SIZE)
        };
        let mut data = Vec::with_capacity(header_bytes.len() + payload.len());
        data.extend_from_slice(header_bytes);
        data.extend_from_slice(payload);
        self.checksum == crc32fast::hash(&data)
    }
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_micros() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_header_size() {
        assert_eq!(std::mem::size_of::<MessageHeader>(), 28);
    }

    #[test]
    fn test_message_header_new() {
        let header = MessageHeader::new(
            MessageType::Data,
            ProtocolType::P2P,
            12345,
            1,
            100,
        );
        
        // Copy values to avoid packed struct alignment issues
        let magic = header.magic;
        let version = header.version;
        let msg_type = header.msg_type;
        let protocol_type = header.protocol_type;
        
        assert_eq!(magic, MessageHeader::MAGIC);
        assert_eq!(version, 0x01);
        assert_eq!(msg_type, MessageType::Data as u8);
        assert_eq!(protocol_type, ProtocolType::P2P as u8);
    }

    #[test]
    fn test_message_header_validate() {
        let mut header = MessageHeader::new(
            MessageType::Handshake,
            ProtocolType::Mother,
            1,
            1,
            0,
        );
        
        assert!(header.validate());
        
        header.magic = 0;
        assert!(!header.validate());
    }

    #[test]
    fn test_checksum() {
        let mut header = MessageHeader::new(
            MessageType::Data,
            ProtocolType::P2P,
            1,
            1,
            10,
        );
        
        let payload = b"test data";
        header.set_checksum(payload);
        
        assert!(header.verify_checksum(payload));
        assert!(!header.verify_checksum(b"wrong data"));
    }
}
