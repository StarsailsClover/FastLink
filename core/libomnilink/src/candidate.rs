//! FastLink ICE candidates module
//!
//! Network candidate gathering and management

use std::net::SocketAddr;
use serde::{Deserialize, Serialize};

/// ICE candidate type
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CandidateType {
    /// Host candidate (direct address)
    Host,
    /// Server reflexive candidate (NAT-mapped)
    ServerReflexive,
    /// Peer reflexive candidate (from peer)
    PeerReflexive,
    /// Relay candidate (TURN)
    Relay,
}

/// ICE candidate priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CandidatePriority(pub u32);

impl CandidatePriority {
    /// Calculate standard ICE priority
    pub fn from_parts(type_preference: u16, local_preference: u16, component: u8) -> Self {
        let priority = (type_preference as u32) << 24
            | (local_preference as u32) << 8
            | (component as u32);
        CandidatePriority(priority)
    }
    
    /// Type preference
    pub fn type_preference(&self) -> u16 {
        ((self.0 >> 24) & 0xFFFF) as u16
    }
    
    /// Local preference
    pub fn local_preference(&self) -> u16 {
        ((self.0 >> 8) & 0xFFFF) as u16
    }
    
    /// Component ID
    pub fn component(&self) -> u8 {
        (self.0 & 0xFF) as u8
    }
}

/// Network protocol type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Protocol {
    Udp,
    Tcp,
}

/// ICE candidate representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IceCandidate {
    /// Candidate type
    pub candidate_type: CandidateType,
    /// Network address
    pub address: SocketAddr,
    /// Base address (host address)
    pub base_address: SocketAddr,
    /// Protocol
    pub protocol: Protocol,
    /// Priority
    pub priority: CandidatePriority,
    /// Component ID
    pub component: u8,
    /// Foundation
    pub foundation: String,
    /// Related address (for reflexive/relay candidates)
    pub related_address: Option<SocketAddr>,
}

impl IceCandidate {
    /// Create a new host candidate
    pub fn new_host(address: SocketAddr, component: u8) -> Self {
        let foundation = format!("host-{}", rand::random::<u32>());
        let priority = CandidatePriority::from_parts(126, 65535, component);
        
        Self {
            candidate_type: CandidateType::Host,
            address,
            base_address: address,
            protocol: Protocol::Udp,
            priority,
            component,
            foundation,
            related_address: None,
        }
    }
    
    /// Create a new server reflexive candidate
    pub fn new_server_reflexive(
        address: SocketAddr,
        base_address: SocketAddr,
        component: u8,
    ) -> Self {
        let foundation = format!("srflx-{}", rand::random::<u32>());
        let priority = CandidatePriority::from_parts(100, 65535, component);
        
        Self {
            candidate_type: CandidateType::ServerReflexive,
            address,
            base_address,
            protocol: Protocol::Udp,
            priority,
            component,
            foundation,
            related_address: Some(base_address),
        }
    }
    
    /// Create a new peer reflexive candidate
    pub fn new_peer_reflexive(
        address: SocketAddr,
        base_address: SocketAddr,
        component: u8,
    ) -> Self {
        let foundation = format!("prflx-{}", rand::random::<u32>());
        let priority = CandidatePriority::from_parts(110, 65535, component);
        
        Self {
            candidate_type: CandidateType::PeerReflexive,
            address,
            base_address,
            protocol: Protocol::Udp,
            priority,
            component,
            foundation,
            related_address: Some(base_address),
        }
    }
    
    /// Create a new relay candidate
    pub fn new_relay(
        address: SocketAddr,
        base_address: SocketAddr,
        component: u8,
    ) -> Self {
        let foundation = format!("relay-{}", rand::random::<u32>());
        let priority = CandidatePriority::from_parts(1, 65535, component);
        
        Self {
            candidate_type: CandidateType::Relay,
            address,
            base_address,
            protocol: Protocol::Udp,
            priority,
            component,
            foundation,
            related_address: Some(base_address),
        }
    }
    
    /// Check if this is a host candidate
    pub fn is_host(&self) -> bool {
        self.candidate_type == CandidateType::Host
    }
    
    /// Check if this is a server reflexive candidate
    pub fn is_server_reflexive(&self) -> bool {
        self.candidate_type == CandidateType::ServerReflexive
    }
    
    /// Check if this is a peer reflexive candidate
    pub fn is_peer_reflexive(&self) -> bool {
        self.candidate_type == CandidateType::PeerReflexive
    }
    
    /// Check if this is a relay candidate
    pub fn is_relay(&self) -> bool {
        self.candidate_type == CandidateType::Relay
    }
}

impl PartialEq for IceCandidate {
    fn eq(&self, other: &Self) -> bool {
        self.address == other.address && self.component == other.component
    }
}

impl Eq for IceCandidate {}

impl PartialOrd for IceCandidate {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for IceCandidate {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.priority.0.cmp(&other.priority.0).reverse()
    }
}

/// Candidate pair for ICE connectivity checks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidatePair {
    /// Local candidate
    pub local: IceCandidate,
    /// Remote candidate
    pub remote: IceCandidate,
    /// Check state
    pub state: CheckState,
    /// Connection priority
    pub connection_priority: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CheckState {
    Waiting,
    InProgress,
    Succeeded,
    Failed,
    Frozen,
}

impl CandidatePair {
    /// Create a new candidate pair
    pub fn new(local: IceCandidate, remote: IceCandidate) -> Self {
        let connection_priority = if local.is_host() && remote.is_host() {
            1000
        } else if local.is_server_reflexive() && remote.is_server_reflexive() {
            900
        } else if local.is_peer_reflexive() || remote.is_peer_reflexive() {
            800
        } else {
            700
        };
        
        Self {
            local,
            remote,
            state: CheckState::Frozen,
            connection_priority,
        }
    }
    
    /// Calculate candidate pair priority
    pub fn calculate_priority(&self) -> u64 {
        let (g, d) = if self.local.priority.0 > self.remote.priority.0 {
            (self.local.priority.0, self.remote.priority.0)
        } else {
            (self.remote.priority.0, self.local.priority.0)
        };
        
        ((u64::MAX >> 1) * g as u64) + (2 * d as u64) + (if self.local.component == 1 { 1 } else { 0 })
    }
}

impl PartialEq for CandidatePair {
    fn eq(&self, other: &Self) -> bool {
        self.local == other.local && self.remote == other.remote
    }
}

impl Eq for CandidatePair {}

impl PartialOrd for CandidatePair {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CandidatePair {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.calculate_priority().cmp(&other.calculate_priority()).reverse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, SocketAddrV4};

    #[test]
    fn test_candidate_priority() {
        let priority = CandidatePriority::from_parts(126, 65535, 1);
        assert_eq!(priority.type_preference(), 126);
        assert_eq!(priority.local_preference(), 65535);
        assert_eq!(priority.component(), 1);
    }

    #[test]
    fn test_candidate_creation() {
        let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(192, 168, 1, 1), 5000));
        let host = IceCandidate::new_host(addr, 1);
        
        assert!(host.is_host());
        assert_eq!(host.component, 1);
    }

    #[test]
    fn test_candidate_ordering() {
        let addr1 = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(192, 168, 1, 1), 5000));
        let addr2 = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(192, 168, 1, 2), 5000));
        
        let host = IceCandidate::new_host(addr1, 1);
        let srflx = IceCandidate::new_server_reflexive(addr2, addr1, 1);
        
        assert!(host > srflx);
    }
}
