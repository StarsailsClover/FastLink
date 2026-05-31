//! Network emulator implementation

use std::collections::VecDeque;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::sync::RwLock;
use tokio::time::{sleep, Instant};
use tracing::info;
use rand::Rng;
use bytes::Bytes;

use crate::models::{Packet, NetworkCondition};
use crate::metrics::NetworkMetrics;

/// Network emulator that applies conditions to packets
pub struct NetworkEmulator {
    /// Incoming packet queue
    incoming: mpsc::Receiver<Packet>,
    /// Outgoing packet queue
    outgoing: mpsc::Sender<Packet>,
    /// Current network conditions
    conditions: Arc<RwLock<NetworkCondition>>,
    /// Metrics collector
    metrics: Arc<RwLock<NetworkMetrics>>,
}

impl NetworkEmulator {
    /// Create a new network emulator
    pub fn new(
        initial_conditions: NetworkCondition,
    ) -> (Self, mpsc::Sender<Packet>, mpsc::Receiver<Packet>) {
        let (incoming_tx, incoming_rx) = mpsc::channel(10_000);
        let (outgoing_tx, outgoing_rx) = mpsc::channel(10_000);
        
        let emulator = Self {
            incoming: incoming_rx,
            outgoing: outgoing_tx,
            conditions: Arc::new(RwLock::new(initial_conditions)),
            metrics: Arc::new(RwLock::new(NetworkMetrics::new())),
        };
        
        (emulator, incoming_tx, outgoing_rx)
    }
    
    /// Start the emulator in a separate task
    pub async fn start(self) {
        let conditions = self.conditions.clone();
        let metrics = self.metrics.clone();
        
        tokio::spawn(async move {
            Self::run(
                self.incoming,
                self.outgoing,
                conditions,
                metrics,
            ).await;
        });
    }
    
    /// Main emulator loop
    async fn run(
        mut incoming: mpsc::Receiver<Packet>,
        outgoing: mpsc::Sender<Packet>,
        conditions: Arc<RwLock<NetworkCondition>>,
        metrics: Arc<RwLock<NetworkMetrics>>,
    ) {
        type PendingPacket = (Instant, Packet);
        let mut pending_packets: VecDeque<PendingPacket> = VecDeque::new();
        
        loop {
            tokio::select! {
                // Handle incoming packets
                packet = incoming.recv() => {
                    if let Some(packet) = packet {
                        // First, record packet without needing RNG
                        metrics.write().await.record_packet(&packet, true);
                        
                        // Now, get conditions and do all RNG operations in one block, no awaits
                        let conds = conditions.read().await.clone();
                        
                        // All RNG usage here, no awaits
                        let (should_drop, should_corrupt, should_duplicate, should_reorder, packet_after_corruption, latency, extra_delay) = {
                            let mut rng = rand::thread_rng();
                            
                            let should_drop = rng.gen::<f64>() < conds.loss_rate;
                            let should_corrupt = rng.gen::<f64>() < conds.corruption_rate;
                            let should_duplicate = rng.gen::<f64>() < conds.duplication_rate;
                            let should_reorder = rng.gen::<f64>() < conds.reorder_rate;
                            
                            let packet_after_corruption = if should_corrupt {
                                Self::corrupt_packet(packet.clone(), &mut rng)
                            } else {
                                packet.clone()
                            };
                            
                            let latency = Self::calculate_latency(&conds, &mut rng);
                            let extra_delay = if should_reorder {
                                Duration::from_millis(rng.gen_range(10..50))
                            } else {
                                Duration::from_millis(0)
                            };
                            
                            (should_drop, should_corrupt, should_duplicate, should_reorder, packet_after_corruption, latency, extra_delay)
                        };
                        
                        if should_drop {
                            metrics.write().await.record_loss();
                            continue;
                        }
                        
                        let final_packet = packet_after_corruption;
                        
                        if should_corrupt {
                            metrics.write().await.record_corruption();
                        }
                        
                        if should_duplicate {
                            let duplicate = final_packet.clone();
                            pending_packets.push_back((Instant::now() + Duration::from_millis(1), duplicate));
                            metrics.write().await.record_duplication();
                        }
                        
                        let deliver_at = Instant::now() + latency;
                        
                        if should_reorder {
                            pending_packets.push_back((deliver_at + extra_delay, final_packet));
                            metrics.write().await.record_reorder();
                        } else {
                            pending_packets.push_back((deliver_at, final_packet));
                        }
                    } else {
                        break;
                    }
                }
                
                // Process pending packets
                _ = async {
                    while let Some((deliver_at, _)) = pending_packets.front() {
                        if *deliver_at <= Instant::now() {
                            if let Some((_, packet)) = pending_packets.pop_front() {
                                if outgoing.send(packet.clone()).await.is_ok() {
                                    metrics.write().await.record_packet(&packet, false);
                                }
                            }
                        } else {
                            break;
                        }
                    }
                    sleep(Duration::from_millis(1)).await;
                } => {}
            }
        }
    }
    
    /// Calculate latency with simple jitter
    fn calculate_latency(conds: &NetworkCondition, rng: &mut impl Rng) -> Duration {
        let base = conds.base_latency;
        let jitter_ms = conds.latency_jitter.as_millis() as u64;
        
        if jitter_ms == 0 {
            return base;
        }
        
        // Simple random jitter
        let jitter = Duration::from_millis(rng.gen_range(0..=jitter_ms));
        base + jitter
    }
    
    /// Corrupt a packet randomly
    fn corrupt_packet(mut packet: Packet, rng: &mut impl Rng) -> Packet {
        let mut bytes = Vec::from(packet.data);
        let corruptions = rng.gen_range(1..=bytes.len().min(10));
        
        for _ in 0..corruptions {
            let idx = rng.gen_range(0..bytes.len());
            bytes[idx] = rng.gen();
        }
        
        packet.data = Bytes::from(bytes);
        packet
    }
    
    /// Update network conditions dynamically
    pub async fn update_conditions(&self, new_conditions: NetworkCondition) {
        info!("Network conditions updated: {:?}", new_conditions);
        let mut conds = self.conditions.write().await;
        *conds = new_conditions;
    }
    
    /// Get current network metrics
    pub async fn get_metrics(&self) -> NetworkMetrics {
        self.metrics.read().await.clone()
    }
    
    /// Reset metrics
    pub async fn reset_metrics(&self) {
        let mut metrics = self.metrics.write().await;
        *metrics = NetworkMetrics::new();
    }
}

/// Virtual network that connects multiple endpoints
pub struct VirtualNetwork {
    endpoints: Arc<RwLock<std::collections::HashMap<SocketAddr, mpsc::Sender<Packet>>>>,
    default_conditions: NetworkCondition,
}

impl VirtualNetwork {
    /// Create a new virtual network
    pub fn new(default_conditions: NetworkCondition) -> Self {
        Self {
            endpoints: Arc::new(RwLock::new(std::collections::HashMap::new())),
            default_conditions,
        }
    }
    
    /// Register a new endpoint
    pub async fn register_endpoint(&self, addr: SocketAddr, sender: mpsc::Sender<Packet>) {
        self.endpoints.write().await.insert(addr, sender);
        info!("Registered endpoint: {}", addr);
    }
    
    /// Send a packet through the virtual network
    pub async fn send_packet(&self, packet: Packet) {
        if let Some(sender) = self.endpoints.read().await.get(&packet.dst) {
            let _ = sender.send(packet).await;
        }
    }
    
    /// Get registered endpoints
    pub async fn get_endpoints(&self) -> Vec<SocketAddr> {
        self.endpoints.read().await.keys().copied().collect()
    }
}
