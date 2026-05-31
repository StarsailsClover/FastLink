//! Network simulation for testing
//!
//! Simulates various network conditions: latency, loss, jitter, reordering.

use std::collections::VecDeque;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::time::{sleep, Instant};

/// Network condition configuration
#[derive(Debug, Clone, Copy)]
pub struct NetworkCondition {
    /// Base latency
    pub latency: Duration,
    /// Latency jitter (random variation)
    pub jitter: Duration,
    /// Packet loss rate (0-100)
    pub loss_rate: f64,
    /// Packet reordering probability (0-100)
    pub reorder_rate: f64,
    /// Bandwidth limit (bytes/sec, 0 = unlimited)
    pub bandwidth: u64,
}

impl NetworkCondition {
    /// Perfect network (no loss, no latency)
    pub fn perfect() -> Self {
        Self {
            latency: Duration::ZERO,
            jitter: Duration::ZERO,
            loss_rate: 0.0,
            reorder_rate: 0.0,
            bandwidth: 0,
        }
    }

    /// Typical WAN condition
    pub fn typical_wan() -> Self {
        Self {
            latency: Duration::from_millis(50),
            jitter: Duration::from_millis(10),
            loss_rate: 0.1,
            reorder_rate: 0.01,
            bandwidth: 10_000_000, // 10 MB/s
        }
    }

    /// Poor network (high loss, high latency)
    pub fn poor() -> Self {
        Self {
            latency: Duration::from_millis(200),
            jitter: Duration::from_millis(50),
            loss_rate: 5.0,
            reorder_rate: 1.0,
            bandwidth: 1_000_000, // 1 MB/s
        }
    }

    /// Mobile network simulation
    pub fn mobile() -> Self {
        Self {
            latency: Duration::from_millis(100),
            jitter: Duration::from_millis(30),
            loss_rate: 2.0,
            reorder_rate: 0.5,
            bandwidth: 5_000_000, // 5 MB/s
        }
    }
}

/// Simulated network link
pub struct SimulatedLink {
    /// Real socket
    socket: UdpSocket,
    /// Peer address
    peer: SocketAddr,
    /// Network conditions
    condition: NetworkCondition,
    /// Pending packets (for latency simulation)
    queue: VecDeque<(Instant, Vec<u8>)>,
    /// Bytes sent in current second (for bandwidth limiting)
    bytes_this_second: u64,
    /// Last bandwidth reset
    last_reset: Instant,
}

impl SimulatedLink {
    /// Create new simulated link
    pub async fn new(
        bind_addr: SocketAddr,
        peer: SocketAddr,
        condition: NetworkCondition,
    ) -> std::io::Result<Self> {
        let socket = UdpSocket::bind(bind_addr).await?;
        
        Ok(Self {
            socket,
            peer,
            condition,
            queue: VecDeque::new(),
            bytes_this_second: 0,
            last_reset: Instant::now(),
        })
    }

    /// Send packet with network simulation
    pub async fn send(&mut self, data: &[u8]) -> std::io::Result<()> {
        // Check bandwidth limit
        if self.condition.bandwidth > 0 {
            if self.last_reset.elapsed() > Duration::from_secs(1) {
                self.bytes_this_second = 0;
                self.last_reset = Instant::now();
            }
            
            self.bytes_this_second += data.len() as u64;
            if self.bytes_this_second > self.condition.bandwidth {
                // Drop packet (bandwidth exceeded)
                return Ok(());
            }
        }

        // Simulate packet loss
        if rand::random::<f64>() < self.condition.loss_rate / 100.0 {
            return Ok(()); // Packet "lost"
        }

        // Calculate delivery time
        let base_latency = self.condition.latency;
        let jitter = if self.condition.jitter > Duration::ZERO {
            let jitter_ms = self.condition.jitter.as_millis() as f64;
            let random_jitter = (rand::random::<f64>() * 2.0 - 1.0) * jitter_ms;
            Duration::from_millis(random_jitter.max(0.0) as u64)
        } else {
            Duration::ZERO
        };
        
        let deliver_at = Instant::now() + base_latency + jitter;

        // Handle reordering
        if rand::random::<f64>() < self.condition.reorder_rate / 100.0 {
            // Add extra delay for reordering
            let reorder_delay = Duration::from_millis(10);
            self.queue.push_back((deliver_at + reorder_delay, data.to_vec()));
        } else {
            self.queue.push_back((deliver_at, data.to_vec()));
        }

        // Process queue
        self.process_queue().await;
        
        Ok(())
    }

    /// Receive packet
    pub async fn recv(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.process_queue().await;
        self.socket.recv_from(buf).await.map(|(n, _)| n)
    }

    /// Process pending packets
    async fn process_queue(&mut self) {
        let now = Instant::now();
        
        while let Some((deliver_at, data)) = self.queue.front() {
            if now >= *deliver_at {
                let (_, data) = self.queue.pop_front().unwrap();
                let _ = self.socket.send_to(&data, self.peer).await;
            } else {
                break;
            }
        }
    }

    /// Get current queue depth
    pub fn queue_depth(&self) -> usize {
        self.queue.len()
    }
}

/// Simulated network for testing multiple nodes
pub struct SimulatedNetwork {
    /// All links
    links: Vec<SimulatedLink>,
    /// Global network condition
    global_condition: NetworkCondition,
}

impl SimulatedNetwork {
    /// Create new simulated network
    pub fn new(condition: NetworkCondition) -> Self {
        Self {
            links: Vec::new(),
            global_condition: condition,
        }
    }

    /// Add node to network
    pub async fn add_node(&mut self, addr: SocketAddr) -> std::io::Result<()> {
        // Create a broadcast socket for this node
        let socket = UdpSocket::bind(addr).await?;
        
        // In a full implementation, this would connect to all other nodes
        // with the simulated network condition
        
        Ok(())
    }

    /// Get network statistics
    pub fn stats(&self) -> NetworkStats {
        NetworkStats {
            active_links: self.links.len(),
            total_queued: self.links.iter().map(|l| l.queue_depth()).sum(),
        }
    }
}

/// Network statistics
#[derive(Debug, Clone, Copy)]
pub struct NetworkStats {
    pub active_links: usize,
    pub total_queued: usize,
}
