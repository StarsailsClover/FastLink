//! FastLink ICE module
//!
//! Interactive Connectivity Establishment (ICE) implementation

use std::collections::{BinaryHeap, HashMap};
use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, Instant};
use thiserror::Error;
use tracing::{debug, info, warn};

use super::candidate::{IceCandidate, CandidatePair, CheckState};
use super::nat::NatProperties;

/// ICE agent state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IceState {
    New,
    Gathering,
    GatheringComplete,
    Connecting,
    Connected,
    Completed,
    Failed,
    Disconnected,
    Closed,
}

/// ICE agent configuration
#[derive(Debug, Clone)]
pub struct IceConfig {
    pub stun_servers: Vec<SocketAddr>,
    pub turn_servers: Vec<TurnServer>,
    pub ice_controlling: bool,
    pub ice_ufrag: String,
    pub ice_pwd: String,
    pub gathering_timeout: Duration,
    pub check_timeout: Duration,
}

impl Default for IceConfig {
    fn default() -> Self {
        Self {
            stun_servers: vec![],
            turn_servers: vec![],
            ice_controlling: true,
            ice_ufrag: format!("fastlink-{}", rand::random::<u32>()),
            ice_pwd: format!("fastlink-{}", rand::random::<u64>()),
            gathering_timeout: Duration::from_secs(5),
            check_timeout: Duration::from_millis(500),
        }
    }
}

/// TURN server configuration
#[derive(Debug, Clone)]
pub struct TurnServer {
    pub address: SocketAddr,
    pub username: String,
    pub password: String,
    pub realm: Option<String>,
}

/// ICE connectivity check result
#[derive(Debug, Clone)]
pub struct ConnectivityCheckResult {
    pub candidate_pair: CandidatePair,
    pub succeeded: bool,
    pub rtt: Option<Duration>,
}

/// ICE agent for managing NAT traversal
pub struct IceAgent {
    state: IceState,
    config: IceConfig,
    local_candidates: Vec<IceCandidate>,
    remote_candidates: Vec<IceCandidate>,
    candidate_pairs: BinaryHeap<CandidatePair>,
    checked_pairs: HashMap<(SocketAddr, SocketAddr), CheckResult>,
    selected_pair: Option<CandidatePair>,
    nat_properties: Option<NatProperties>,
}

#[derive(Debug, Clone)]
struct CheckResult {
    succeeded: bool,
    rtt: Option<Duration>,
    timestamp: Instant,
}

impl IceAgent {
    /// Create a new ICE agent
    pub fn new(config: IceConfig) -> Self {
        Self {
            state: IceState::New,
            config,
            local_candidates: Vec::new(),
            remote_candidates: Vec::new(),
            candidate_pairs: BinaryHeap::new(),
            checked_pairs: HashMap::new(),
            selected_pair: None,
            nat_properties: None,
        }
    }
    
    /// Get current ICE state
    pub fn state(&self) -> IceState {
        self.state
    }
    
    /// Add a local candidate
    pub fn add_local_candidate(&mut self, candidate: IceCandidate) {
        debug!("Adding local candidate: {:?}", candidate);
        self.local_candidates.push(candidate);
        self.update_candidate_pairs();
    }
    
    /// Add remote candidates
    pub fn add_remote_candidates(&mut self, candidates: Vec<IceCandidate>) {
        debug!("Adding {} remote candidates", candidates.len());
        self.remote_candidates.extend(candidates);
        self.update_candidate_pairs();
    }
    
    /// Update candidate pairs
    fn update_candidate_pairs(&mut self) {
        for local in &self.local_candidates {
            for remote in &self.remote_candidates {
                let pair = CandidatePair::new(local.clone(), remote.clone());
                if !self.candidate_pairs.iter().any(|p| p == &pair) {
                    self.candidate_pairs.push(pair);
                }
            }
        }
    }
    
    /// Get local candidates
    pub fn local_candidates(&self) -> &[IceCandidate] {
        &self.local_candidates
    }
    
    /// Get remote candidates
    pub fn remote_candidates(&self) -> &[IceCandidate] {
        &self.remote_candidates
    }
    
    /// Start gathering candidates
    pub async fn start_gathering(&mut self) -> Result<(), IceError> {
        self.state = IceState::Gathering;
        
        // Gather host candidates
        self.gather_host_candidates()?;
        
        // Gather server reflexive candidates using STUN
        // Clone the STUN servers list to avoid borrowing issues
        let stun_servers = self.config.stun_servers.clone();
        
        for stun_server in stun_servers {
            if let Err(e) = self.gather_server_reflexive_candidates(stun_server).await {
                warn!("Failed to gather STUN candidates from {}: {}", stun_server, e);
            }
        }
        
        self.state = IceState::GatheringComplete;
        Ok(())
    }
    
    /// Gather host candidates
    fn gather_host_candidates(&mut self) -> Result<(), IceError> {
        // Get local network interfaces
        let interfaces = if_addrs::get_if_addrs()?;
        
        for iface in interfaces {
            if iface.is_loopback() {
                continue;
            }
            
            let addr = SocketAddr::new(iface.ip(), 0);
            
            // Try to bind to this address
            if let Ok(socket) = UdpSocket::bind(addr) {
                if let Ok(local_addr) = socket.local_addr() {
                    let candidate = IceCandidate::new_host(local_addr, 1);
                    self.add_local_candidate(candidate);
                    debug!("Host candidate added: {}", local_addr);
                }
            }
        }
        
        Ok(())
    }
    
    /// Gather server reflexive candidates
    async fn gather_server_reflexive_candidates(&mut self, stun_server: SocketAddr) -> Result<(), IceError> {
        // Create a socket for STUN
        let local_addr = SocketAddr::new(std::net::Ipv4Addr::UNSPECIFIED.into(), 0);
        let socket = UdpSocket::bind(local_addr)?;
        
        let stun_client = super::stun::StunClient::new(socket.local_addr()?, stun_server)?;
        
        if let Ok(public_addr) = stun_client.query_public_address() {
            let base_addr = socket.local_addr()?;
            let candidate = IceCandidate::new_server_reflexive(public_addr, base_addr, 1);
            self.add_local_candidate(candidate);
            info!("Server reflexive candidate added: {}", public_addr);
        }
        
        Ok(())
    }
    
    /// Perform connectivity checks
    pub async fn perform_connectivity_checks(&mut self) -> Result<(), IceError> {
        self.state = IceState::Connecting;
        
        let start_time = Instant::now();
        
        while self.state == IceState::Connecting {
            if start_time.elapsed() > self.config.check_timeout * 10 {
                self.state = IceState::Failed;
                return Err(IceError::ConnectivityCheckTimeout);
            }
            
            // Get next candidate pair to check
            if let Some(mut pair) = self.candidate_pairs.pop() {
                if pair.state != CheckState::Frozen && pair.state != CheckState::Waiting {
                    continue;
                }
                
                // Perform the check
                let result = self.check_pair(&pair).await;
                
                if result.succeeded {
                    pair.state = CheckState::Succeeded;
                    self.selected_pair = Some(pair.clone());
                    self.state = IceState::Connected;
                    info!("Connectivity check succeeded: {:?}", result);
                    break;
                } else {
                    pair.state = CheckState::Failed;
                }
                
                self.checked_pairs.insert(
                    (pair.local.address, pair.remote.address),
                    CheckResult {
                        succeeded: result.succeeded,
                        rtt: result.rtt,
                        timestamp: Instant::now(),
                    },
                );
            } else {
                // No more candidate pairs to check
                self.state = IceState::Failed;
                return Err(IceError::NoWorkingCandidate);
            }
        }
        
        Ok(())
    }
    
    /// Check a single candidate pair
    async fn check_pair(&self, pair: &CandidatePair) -> ConnectivityCheckResult {
        let start_time = Instant::now();
        
        // Create a socket for this check
        if let Ok(socket) = UdpSocket::bind(pair.local.address) {
            // Try to set non-blocking mode, but don't fail if we can't
            let _ = socket.set_nonblocking(true);
            
            // Send a simple ping packet
            let ping_data = format!("FASTLINK-PING-{}", rand::random::<u64>());
            
            if socket.send_to(ping_data.as_bytes(), pair.remote.address).is_ok() {
                // Wait for a response
                let mut buf = [0u8; 1024];
                
                let mut wait_time = Duration::from_millis(100);
                for _ in 0..5 {
                    tokio::time::sleep(wait_time).await;
                    
                    if let Ok((size, from)) = socket.recv_from(&mut buf) {
                        if from == pair.remote.address {
                            let rtt = start_time.elapsed();
                            return ConnectivityCheckResult {
                                candidate_pair: pair.clone(),
                                succeeded: true,
                                rtt: Some(rtt),
                            };
                        }
                    }
                    
                    wait_time *= 2;
                }
            }
        }
        
        ConnectivityCheckResult {
            candidate_pair: pair.clone(),
            succeeded: false,
            rtt: None,
        }
    }
    
    /// Get selected candidate pair
    pub fn selected_pair(&self) -> Option<&CandidatePair> {
        self.selected_pair.as_ref()
    }
    
    /// Close the ICE agent
    pub fn close(&mut self) {
        self.state = IceState::Closed;
    }
}

#[derive(Debug, Error)]
pub enum IceError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("STUN error: {0}")]
    Stun(#[from] super::stun::StunError),
    #[error("Connectivity check timeout")]
    ConnectivityCheckTimeout,
    #[error("No working candidate found")]
    NoWorkingCandidate,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, SocketAddrV4};

    #[test]
    fn test_ice_agent_creation() {
        let config = IceConfig::default();
        let agent = IceAgent::new(config);
        
        assert_eq!(agent.state(), IceState::New);
        assert!(agent.local_candidates().is_empty());
    }

    #[test]
    fn test_ice_candidate_pair_creation() {
        let addr1 = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(192, 168, 1, 1), 5000));
        let addr2 = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(192, 168, 1, 2), 5000));
        
        let local = IceCandidate::new_host(addr1, 1);
        let remote = IceCandidate::new_host(addr2, 1);
        
        let pair = CandidatePair::new(local, remote);
        assert_eq!(pair.state, CheckState::Frozen);
    }
}
