//! Network models and packet definitions

use std::net::SocketAddr;
use std::time::Duration;
use bytes::Bytes;

/// Network packet being routed through the emulator
#[derive(Debug, Clone)]
pub struct Packet {
    /// Packet data
    pub data: Bytes,
    /// Source address
    pub src: SocketAddr,
    /// Destination address
    pub dst: SocketAddr,
    /// Timestamp when packet was sent
    pub sent_at: Duration,
    /// Unique packet identifier
    pub id: u64,
}

/// Network condition configuration
#[derive(Debug, Clone)]
pub struct NetworkCondition {
    /// Packet loss rate (0.0 - 1.0)
    pub loss_rate: f64,
    /// Packet corruption rate (0.0 - 1.0)
    pub corruption_rate: f64,
    /// Base latency
    pub base_latency: Duration,
    /// Latency jitter (standard deviation)
    pub latency_jitter: Duration,
    /// Downstream bandwidth (bytes per second)
    pub bandwidth_down: u64,
    /// Upstream bandwidth (bytes per second)
    pub bandwidth_up: u64,
    /// Packet reorder rate (0.0 - 1.0)
    pub reorder_rate: f64,
    /// Max reorder buffer size
    pub max_reorder_buffer: usize,
    /// Packet duplication rate (0.0 - 1.0)
    pub duplication_rate: f64,
}

impl Default for NetworkCondition {
    fn default() -> Self {
        Self {
            loss_rate: 0.0,
            corruption_rate: 0.0,
            base_latency: Duration::from_millis(50),
            latency_jitter: Duration::from_millis(5),
            bandwidth_down: 100_000_000, // 100 Mbps
            bandwidth_up: 100_000_000,   // 100 Mbps
            reorder_rate: 0.0,
            max_reorder_buffer: 100,
            duplication_rate: 0.0,
        }
    }
}

/// Predefined network scenarios
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkScenario {
    /// Perfect network conditions
    Perfect,
    /// Typical home broadband
    HomeBroadband,
    /// Mobile 3G network
    Mobile3G,
    /// Mobile 4G/LTE network
    Mobile4G,
    /// Mobile 5G network
    Mobile5G,
    /// Satellite connection
    Satellite,
    /// Rural slow connection
    RuralSlow,
    /// Intermittent connection
    Intermittent,
    /// High latency connection
    HighLatency,
    /// Congested network
    Congested,
}

impl NetworkScenario {
    /// Get network conditions for a scenario
    pub fn conditions(&self) -> NetworkCondition {
        match self {
            Self::Perfect => NetworkCondition {
                loss_rate: 0.0,
                corruption_rate: 0.0,
                base_latency: Duration::from_millis(1),
                latency_jitter: Duration::from_millis(0),
                bandwidth_down: 1_000_000_000, // 1 Gbps
                bandwidth_up: 1_000_000_000,   // 1 Gbps
                reorder_rate: 0.0,
                max_reorder_buffer: 0,
                duplication_rate: 0.0,
            },
            Self::HomeBroadband => NetworkCondition {
                loss_rate: 0.001,
                corruption_rate: 0.0001,
                base_latency: Duration::from_millis(20),
                latency_jitter: Duration::from_millis(3),
                bandwidth_down: 100_000_000, // 100 Mbps
                bandwidth_up: 20_000_000,   // 20 Mbps
                reorder_rate: 0.01,
                max_reorder_buffer: 20,
                duplication_rate: 0.001,
            },
            Self::Mobile3G => NetworkCondition {
                loss_rate: 0.03,
                corruption_rate: 0.005,
                base_latency: Duration::from_millis(200),
                latency_jitter: Duration::from_millis(50),
                bandwidth_down: 384_000, // 384 Kbps
                bandwidth_up: 128_000,  // 128 Kbps
                reorder_rate: 0.05,
                max_reorder_buffer: 100,
                duplication_rate: 0.01,
            },
            Self::Mobile4G => NetworkCondition {
                loss_rate: 0.005,
                corruption_rate: 0.001,
                base_latency: Duration::from_millis(30),
                latency_jitter: Duration::from_millis(10),
                bandwidth_down: 50_000_000, // 50 Mbps
                bandwidth_up: 10_000_000,   // 10 Mbps
                reorder_rate: 0.02,
                max_reorder_buffer: 50,
                duplication_rate: 0.005,
            },
            Self::Mobile5G => NetworkCondition {
                loss_rate: 0.001,
                corruption_rate: 0.0001,
                base_latency: Duration::from_millis(10),
                latency_jitter: Duration::from_millis(2),
                bandwidth_down: 1_000_000_000, // 1 Gbps
                bandwidth_up: 200_000_000,  // 200 Mbps
                reorder_rate: 0.005,
                max_reorder_buffer: 30,
                duplication_rate: 0.001,
            },
            Self::Satellite => NetworkCondition {
                loss_rate: 0.02,
                corruption_rate: 0.003,
                base_latency: Duration::from_millis(600),
                latency_jitter: Duration::from_millis(100),
                bandwidth_down: 20_000_000, // 20 Mbps
                bandwidth_up: 5_000_000,  // 5 Mbps
                reorder_rate: 0.03,
                max_reorder_buffer: 200,
                duplication_rate: 0.01,
            },
            Self::RuralSlow => NetworkCondition {
                loss_rate: 0.05,
                corruption_rate: 0.01,
                base_latency: Duration::from_millis(400),
                latency_jitter: Duration::from_millis(80),
                bandwidth_down: 1_000_000, // 1 Mbps
                bandwidth_up: 256_000,  // 256 Kbps
                reorder_rate: 0.08,
                max_reorder_buffer: 150,
                duplication_rate: 0.02,
            },
            Self::Intermittent => NetworkCondition {
                loss_rate: 0.1,
                corruption_rate: 0.01,
                base_latency: Duration::from_millis(150),
                latency_jitter: Duration::from_millis(100),
                bandwidth_down: 10_000_000, // 10 Mbps
                bandwidth_up: 2_000_000,   // 2 Mbps
                reorder_rate: 0.05,
                max_reorder_buffer: 80,
                duplication_rate: 0.01,
            },
            Self::HighLatency => NetworkCondition {
                loss_rate: 0.01,
                corruption_rate: 0.002,
                base_latency: Duration::from_millis(500),
                latency_jitter: Duration::from_millis(150),
                bandwidth_down: 50_000_000, // 50 Mbps
                bandwidth_up: 10_000_000,  // 10 Mbps
                reorder_rate: 0.02,
                max_reorder_buffer: 200,
                duplication_rate: 0.005,
            },
            Self::Congested => NetworkCondition {
                loss_rate: 0.08,
                corruption_rate: 0.005,
                base_latency: Duration::from_millis(300),
                latency_jitter: Duration::from_millis(200),
                bandwidth_down: 5_000_000, // 5 Mbps
                bandwidth_up: 1_000_000,   // 1 Mbps
                reorder_rate: 0.1,
                max_reorder_buffer: 300,
                duplication_rate: 0.02,
            },
        }
    }
}

/// Network endpoint configuration
#[derive(Debug, Clone)]
pub struct EndpointConfig {
    /// Listen address
    pub listen_addr: SocketAddr,
    /// Network conditions specific to this endpoint
    pub outgoing_condition: Option<NetworkCondition>,
    pub incoming_condition: Option<NetworkCondition>,
}
