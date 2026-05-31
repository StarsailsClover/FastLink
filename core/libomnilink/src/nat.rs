//! FastLink NAT detection module
//!
//! NAT type detection and characterization

use std::net::{SocketAddr, Ipv4Addr};
use thiserror::Error;
use serde::{Serialize, Deserialize};
use super::stun::{StunClient, StunError};
use tracing::{debug, info};

/// NAT type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NatType {
    /// Open Internet (no NAT)
    Open,
    /// Full cone NAT
    FullCone,
    /// Restricted cone NAT
    RestrictedCone,
    /// Port restricted cone NAT
    PortRestrictedCone,
    /// Symmetric NAT
    Symmetric,
    /// Unknown NAT type
    Unknown,
}

impl std::fmt::Display for NatType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NatType::Open => write!(f, "Open (No NAT)"),
            NatType::FullCone => write!(f, "Full Cone NAT"),
            NatType::RestrictedCone => write!(f, "Restricted Cone NAT"),
            NatType::PortRestrictedCone => write!(f, "Port Restricted Cone NAT"),
            NatType::Symmetric => write!(f, "Symmetric NAT"),
            NatType::Unknown => write!(f, "Unknown NAT type"),
        }
    }
}

/// NAT properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatProperties {
    pub nat_type: NatType,
    pub public_address: Option<SocketAddr>,
    pub local_address: Option<SocketAddr>,
    pub filter_only_allow_src_port: bool,
    pub filter_only_allow_src_ip: bool,
    pub mapping_uses_dest_ip: bool,
    pub mapping_uses_dest_port: bool,
    pub hairpin_support: bool,
}

impl NatProperties {
    pub fn new() -> Self {
        Self {
            nat_type: NatType::Unknown,
            public_address: None,
            local_address: None,
            filter_only_allow_src_port: false,
            filter_only_allow_src_ip: false,
            mapping_uses_dest_ip: false,
            mapping_uses_dest_port: false,
            hairpin_support: false,
        }
    }
}

impl Default for NatProperties {
    fn default() -> Self {
        Self::new()
    }
}

/// NAT detector using STUN
pub struct NatDetector {
    stun_server1: SocketAddr,
    stun_server2: SocketAddr,
}

impl NatDetector {
    /// Create a new NAT detector
    pub fn new(stun_server1: SocketAddr, stun_server2: SocketAddr) -> Self {
        Self {
            stun_server1,
            stun_server2,
        }
    }
    
    /// Detect NAT type using RFC 3489 method
    pub async fn detect(&self, local_port: u16) -> Result<NatProperties, NatDetectionError> {
        let mut properties = NatProperties::new();
        
        // Get public address from first STUN server
        let local_addr1 = SocketAddr::new(
            std::net::IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            local_port,
        );
        
        let client1 = StunClient::new(local_addr1, self.stun_server1)?;
        let public_addr1 = client1.query_public_address()?;
        properties.public_address = Some(public_addr1);
        properties.local_address = Some(local_addr1);
        
        info!("Public address: {}", public_addr1);
        
        // Check if we have a public address (no NAT)
        if Self::is_public_ip(public_addr1.ip()) {
            properties.nat_type = NatType::Open;
            info!("Detected: {}", properties.nat_type);
            return Ok(properties);
        }
        
        // Query second STUN server for same local port
        let local_addr2 = SocketAddr::new(
            std::net::IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            local_port,
        );
        
        let client2 = StunClient::new(local_addr2, self.stun_server2)?;
        let public_addr2 = client2.query_public_address()?;
        
        debug!("STUN server 1: {}, STUN server 2: {}", self.stun_server1, self.stun_server2);
        debug!("Public address 1: {}, Public address 2: {}", public_addr1, public_addr2);
        
        // If addresses are the same, it's a cone NAT
        if public_addr1.ip() == public_addr2.ip() && public_addr1.port() == public_addr2.port() {
            properties.mapping_uses_dest_ip = false;
            properties.mapping_uses_dest_port = false;
            
            // Now determine filtering type (restricted cone, port restricted, full)
            // This requires a more comprehensive test with multiple peers
            // For now, we'll default to full cone
            properties.nat_type = NatType::FullCone;
            
            info!("Detected: {}", properties.nat_type);
        } else {
            // Addresses are different - symmetric NAT or uses destination info
            properties.mapping_uses_dest_ip = true;
            properties.mapping_uses_dest_port = true;
            properties.nat_type = NatType::Symmetric;
            
            info!("Detected: {}", properties.nat_type);
        }
        
        Ok(properties)
    }
    
    /// Check if an IP address is public
    fn is_public_ip(ip: std::net::IpAddr) -> bool {
        match ip {
            std::net::IpAddr::V4(v4) => {
                !v4.is_private() 
                    && !v4.is_loopback() 
                    && !v4.is_link_local() 
                    && !v4.is_multicast()
                    && !v4.is_broadcast()
                    && !v4.is_unspecified()
            }
            std::net::IpAddr::V6(v6) => {
                !v6.is_loopback() 
                    && !v6.is_multicast() 
                    && !v6.is_unspecified()
                    && !v6.is_unicast_link_local()
            }
        }
    }
    
    /// Check if NAT traversal is possible without TURN
    pub fn is_direct_traversal_possible(local: &NatProperties, remote: &NatProperties) -> bool {
        match (local.nat_type, remote.nat_type) {
            // Open <-> anything
            (NatType::Open, _) => true,
            (_, NatType::Open) => true,
            
            // Full cone to anything
            (NatType::FullCone, _) => true,
            (_, NatType::FullCone) => true,
            
            // Restricted cones can connect in certain configurations
            (NatType::RestrictedCone, NatType::RestrictedCone) => true,
            (NatType::RestrictedCone, NatType::PortRestrictedCone) => true,
            (NatType::PortRestrictedCone, NatType::RestrictedCone) => true,
            (NatType::PortRestrictedCone, NatType::PortRestrictedCone) => true,
            
            // Symmetric NAT connections are hard
            (NatType::Symmetric, NatType::Symmetric) => false,
            (NatType::Symmetric, _) => false,
            (_, NatType::Symmetric) => false,
            
            _ => false,
        }
    }
    
    /// Check if TURN relay is required
    pub fn requires_turn(local: &NatProperties, remote: &NatProperties) -> bool {
        !Self::is_direct_traversal_possible(local, remote)
    }
}

#[derive(Debug, Error)]
pub enum NatDetectionError {
    #[error("STUN error: {0}")]
    Stun(#[from] StunError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Detection timeout")]
    Timeout,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nat_type_display() {
        assert_eq!(NatType::Open.to_string(), "Open (No NAT)");
        assert_eq!(NatType::FullCone.to_string(), "Full Cone NAT");
    }

    #[test]
    fn test_nat_properties_default() {
        let props = NatProperties::default();
        assert_eq!(props.nat_type, NatType::Unknown);
        assert!(props.public_address.is_none());
    }
}
