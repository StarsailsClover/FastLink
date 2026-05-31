//! Metrics collection and analysis

use std::collections::VecDeque;
use std::time::Duration;

use crate::models::Packet;

/// Network performance metrics
#[derive(Debug, Clone)]
pub struct NetworkMetrics {
    /// Total packets sent
    pub total_sent: u64,
    /// Total packets received
    pub total_received: u64,
    /// Number of packets lost
    pub lost_packets: u64,
    /// Number of packets corrupted
    pub corrupted_packets: u64,
    /// Number of packets reordered
    pub reordered_packets: u64,
    /// Number of packets duplicated
    pub duplicated_packets: u64,
    /// Total bytes sent
    pub bytes_sent: u64,
    /// Total bytes received
    pub bytes_received: u64,
    /// Packet loss rate (0.0 - 1.0)
    pub loss_rate: f64,
    /// Latency samples (milliseconds)
    pub latency_samples: VecDeque<f64>,
    /// Start time of metrics collection
    pub start_time: Duration,
    /// List of message round trip times (RTT)
    pub rtt_samples: VecDeque<f64>,
}

impl NetworkMetrics {
    /// Create a new empty metrics object
    pub fn new() -> Self {
        Self {
            total_sent: 0,
            total_received: 0,
            lost_packets: 0,
            corrupted_packets: 0,
            reordered_packets: 0,
            duplicated_packets: 0,
            bytes_sent: 0,
            bytes_received: 0,
            loss_rate: 0.0,
            latency_samples: VecDeque::with_capacity(1000),
            start_time: Duration::from_millis(0),
            rtt_samples: VecDeque::with_capacity(1000),
        }
    }
    
    /// Record a packet
    pub fn record_packet(&mut self, packet: &Packet, is_sent: bool) {
        if is_sent {
            self.total_sent += 1;
            self.bytes_sent += packet.data.len() as u64;
        } else {
            self.total_received += 1;
            self.bytes_received += packet.data.len() as u64;
        }
        
        // Update loss rate
        if self.total_sent > 0 {
            self.loss_rate = self.lost_packets as f64 / self.total_sent as f64;
        }
    }
    
    /// Record a lost packet
    pub fn record_loss(&mut self) {
        self.lost_packets += 1;
        if self.total_sent > 0 {
            self.loss_rate = self.lost_packets as f64 / self.total_sent as f64;
        }
    }
    
    /// Record a corrupted packet
    pub fn record_corruption(&mut self) {
        self.corrupted_packets += 1;
    }
    
    /// Record a reordered packet
    pub fn record_reorder(&mut self) {
        self.reordered_packets += 1;
    }
    
    /// Record a duplicated packet
    pub fn record_duplication(&mut self) {
        self.duplicated_packets += 1;
    }
    
    /// Record a latency sample
    pub fn record_latency(&mut self, latency_ms: f64) {
        self.latency_samples.push_back(latency_ms);
        if self.latency_samples.len() > 1000 {
            self.latency_samples.pop_front();
        }
    }
    
    /// Record an RTT sample
    pub fn record_rtt(&mut self, rtt_ms: f64) {
        self.rtt_samples.push_back(rtt_ms);
        if self.rtt_samples.len() > 1000 {
            self.rtt_samples.pop_front();
        }
    }
    
    /// Calculate average latency in milliseconds
    pub fn avg_latency(&self) -> Option<f64> {
        if self.latency_samples.is_empty() {
            return None;
        }
        
        let sum: f64 = self.latency_samples.iter().sum();
        Some(sum / self.latency_samples.len() as f64)
    }
    
    /// Calculate latency percentile (simple implementation)
    pub fn latency_percentile(&self, percentile: f64) -> Option<f64> {
        if self.latency_samples.is_empty() {
            return None;
        }
        
        let mut samples: Vec<_> = self.latency_samples.iter().cloned().collect();
        samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let idx = (percentile / 100.0 * (samples.len() - 1) as f64).round() as usize;
        Some(samples[idx])
    }
    
    /// Calculate average RTT in milliseconds
    pub fn avg_rtt(&self) -> Option<f64> {
        if self.rtt_samples.is_empty() {
            return None;
        }
        
        let sum: f64 = self.rtt_samples.iter().sum();
        Some(sum / self.rtt_samples.len() as f64)
    }
    
    /// Calculate RTT percentile (simple implementation)
    pub fn rtt_percentile(&self, percentile: f64) -> Option<f64> {
        if self.rtt_samples.is_empty() {
            return None;
        }
        
        let mut samples: Vec<_> = self.rtt_samples.iter().cloned().collect();
        samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let idx = (percentile / 100.0 * (samples.len() - 1) as f64).round() as usize;
        Some(samples[idx])
    }
    
    /// Get summary statistics
    pub fn summary(&self) -> MetricsSummary {
        MetricsSummary {
            total_sent: self.total_sent,
            total_received: self.total_received,
            loss_rate: self.loss_rate,
            avg_latency_ms: self.avg_latency(),
            p50_latency_ms: self.latency_percentile(50.0),
            p90_latency_ms: self.latency_percentile(90.0),
            p95_latency_ms: self.latency_percentile(95.0),
            p99_latency_ms: self.latency_percentile(99.0),
            avg_rtt_ms: self.avg_rtt(),
            corrupted_packets: self.corrupted_packets,
            reordered_packets: self.reordered_packets,
            duplicated_packets: self.duplicated_packets,
            bytes_sent: self.bytes_sent,
            bytes_received: self.bytes_received,
        }
    }
}

impl Default for NetworkMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Metrics summary report
#[derive(Debug, Clone)]
pub struct MetricsSummary {
    pub total_sent: u64,
    pub total_received: u64,
    pub loss_rate: f64,
    pub avg_latency_ms: Option<f64>,
    pub p50_latency_ms: Option<f64>,
    pub p90_latency_ms: Option<f64>,
    pub p95_latency_ms: Option<f64>,
    pub p99_latency_ms: Option<f64>,
    pub avg_rtt_ms: Option<f64>,
    pub corrupted_packets: u64,
    pub reordered_packets: u64,
    pub duplicated_packets: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
}

impl MetricsSummary {
    /// Print a human-readable summary
    pub fn print(&self) {
        println!("=== Network Metrics Summary ===");
        println!("Total sent: {}", self.total_sent);
        println!("Total received: {}", self.total_received);
        println!("Loss rate: {:.2}%", self.loss_rate * 100.0);
        println!("Corrupted packets: {}", self.corrupted_packets);
        println!("Reordered packets: {}", self.reordered_packets);
        println!("Duplicated packets: {}", self.duplicated_packets);
        println!("Bytes sent: {}", self.bytes_sent);
        println!("Bytes received: {}", self.bytes_received);
        
        if let Some(avg) = self.avg_latency_ms {
            println!("Average latency: {:.2} ms", avg);
        }
        
        if let Some(p50) = self.p50_latency_ms {
            println!("p50 latency: {:.2} ms", p50);
        }
        
        if let Some(p90) = self.p90_latency_ms {
            println!("p90 latency: {:.2} ms", p90);
        }
        
        if let Some(p95) = self.p95_latency_ms {
            println!("p95 latency: {:.2} ms", p95);
        }
        
        if let Some(p99) = self.p99_latency_ms {
            println!("p99 latency: {:.2} ms", p99);
        }
        
        if let Some(rtt) = self.avg_rtt_ms {
            println!("Average RTT: {:.2} ms", rtt);
        }
        
        println!("==============================");
    }
}

/// Metrics collector trait
pub trait MetricsCollector: Send + Sync {
    fn record_packet(&mut self, packet: &Packet, is_sent: bool);
    fn record_loss(&mut self);
    fn record_latency(&mut self, latency_ms: f64);
    fn record_rtt(&mut self, rtt_ms: f64);
    fn get_summary(&self) -> MetricsSummary;
}

impl MetricsCollector for NetworkMetrics {
    fn record_packet(&mut self, packet: &Packet, is_sent: bool) {
        self.record_packet(packet, is_sent);
    }
    
    fn record_loss(&mut self) {
        self.record_loss();
    }
    
    fn record_latency(&mut self, latency_ms: f64) {
        self.record_latency(latency_ms);
    }
    
    fn record_rtt(&mut self, rtt_ms: f64) {
        self.record_rtt(rtt_ms);
    }
    
    fn get_summary(&self) -> MetricsSummary {
        self.summary()
    }
}
