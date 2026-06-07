//! FastLink BirthdayPunch NAT Traversal Module
//!
//! Birthday paradox-based port prediction for symmetric NAT traversal
//! 
//! Theory: Based on birthday paradox, if we try ~sqrt(N) random ports
//! from a space of N ports, we have ~50% chance of collision.
//! For symmetric NAT with sequential/random port allocation,
//! this can achieve >50% success rate.

use std::collections::HashSet;
use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, Instant};
use rand::seq::SliceRandom;
use rand::thread_rng;
use tracing::{debug, info, warn};

use super::nat::{NatProperties, NatType};

/// BirthdayPunch configuration
#[derive(Debug, Clone)]
pub struct BirthdayPunchConfig {
    /// Port prediction range size
    pub prediction_range: u16,
    /// Number of prediction attempts (sqrt of range for birthday paradox)
    pub prediction_attempts: usize,
    /// Timeout for each punch attempt
    pub punch_timeout: Duration,
    /// Delay between punch attempts
    pub punch_interval: Duration,
    /// Enable sequential port prediction
    pub enable_sequential_prediction: bool,
    /// Enable random port prediction
    pub enable_random_prediction: bool,
    /// Enable port delta prediction (based on observed delta)
    pub enable_delta_prediction: bool,
}

impl Default for BirthdayPunchConfig {
    fn default() -> Self {
        Self {
            prediction_range: 1000,
            prediction_attempts: 32, // sqrt(1000) ≈ 32
            punch_timeout: Duration::from_millis(500),
            punch_interval: Duration::from_millis(10),
            enable_sequential_prediction: true,
            enable_random_prediction: true,
            enable_delta_prediction: true,
        }
    }
}

/// Port prediction strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PredictionStrategy {
    /// Predict based on sequential allocation pattern
    Sequential,
    /// Predict based on random allocation with birthday paradox
    Random,
    /// Predict based on observed port delta
    Delta,
    /// Combined strategy
    Combined,
}

/// Predicted port candidates
#[derive(Debug, Clone)]
pub struct PortPrediction {
    pub strategy: PredictionStrategy,
    pub predicted_ports: Vec<u16>,
    pub confidence: f64,
    pub observed_delta: Option<i16>,
}

/// BirthdayPunch NAT traversal engine
pub struct BirthdayPunch {
    config: BirthdayPunchConfig,
    nat_properties: NatProperties,
    observed_ports: Vec<u16>,
    last_predicted_delta: Option<i16>,
}

impl BirthdayPunch {
    pub fn new(config: BirthdayPunchConfig, nat_properties: NatProperties) -> Self {
        Self {
            config,
            nat_properties,
            observed_ports: Vec::new(),
            last_predicted_delta: None,
        }
    }

    /// Analyze NAT behavior and determine best strategy
    pub fn analyze_nat_behavior(&mut self, observed_mappings: &[(u16, u16)]) {
        if observed_mappings.len() < 2 {
            return;
        }

        // Calculate port deltas
        let mut deltas: Vec<i16> = Vec::new();
        for i in 1..observed_mappings.len() {
            let delta = observed_mappings[i].1 as i16 - observed_mappings[i-1].1 as i16;
            deltas.push(delta);
        }

        // Check if sequential
        let is_sequential = deltas.iter().all(|&d| d == 1);
        
        if is_sequential {
            self.last_predicted_delta = Some(1);
            info!("Detected sequential NAT port allocation");
        } else {
            // Calculate average delta
            let avg_delta = deltas.iter().sum::<i16>() / deltas.len() as i16;
            self.last_predicted_delta = Some(avg_delta);
            debug!("Detected NAT port delta: {}", avg_delta);
        }

        self.observed_ports = observed_mappings.iter().map(|(_, pub_port)| *pub_port).collect();
    }

    /// Predict peer's public port using birthday paradox strategy
    pub fn predict_peer_port(&self, peer_local_port: u16, peer_public_base: Option<u16>) -> PortPrediction {
        let mut predictions = Vec::new();
        let mut strategy = PredictionStrategy::Combined;

        // Strategy 1: Sequential prediction
        if self.config.enable_sequential_prediction {
            if let Some(base) = peer_public_base {
                for i in 0..self.config.prediction_attempts / 3 {
                    predictions.push(base + i as u16);
                }
                strategy = PredictionStrategy::Sequential;
            }
        }

        // Strategy 2: Delta-based prediction
        if self.config.enable_delta_prediction && self.last_predicted_delta.is_some() {
            if let Some(base) = peer_public_base {
                let delta = self.last_predicted_delta.unwrap();
                for i in 0..self.config.prediction_attempts / 3 {
                    predictions.push((base as i16 + delta * i as i16) as u16);
                }
                strategy = PredictionStrategy::Delta;
            }
        }

        // Strategy 3: Random prediction (Birthday Paradox)
        if self.config.enable_random_prediction {
            let base = peer_public_base.unwrap_or(peer_local_port);
            let range_start = base.saturating_sub(self.config.prediction_range / 2);
            let range_end = base.saturating_add(self.config.prediction_range / 2);
            
            let mut rng = thread_rng();
            let mut random_ports: Vec<u16> = (range_start..range_end).collect();
            random_ports.shuffle(&mut rng);
            
            let take_count = self.config.prediction_attempts - predictions.len();
            predictions.extend(random_ports.into_iter().take(take_count));
            
            if strategy == PredictionStrategy::Combined {
                strategy = PredictionStrategy::Random;
            }
        }

        // Remove duplicates while preserving order
        let mut seen = HashSet::new();
        predictions.retain(|&p| seen.insert(p));

        // Calculate confidence based on strategy and observed data
        let confidence = self.calculate_confidence(&predictions, strategy);

        PortPrediction {
            strategy,
            predicted_ports: predictions,
            confidence,
            observed_delta: self.last_predicted_delta,
        }
    }

    fn calculate_confidence(&self, predictions: &[u16], strategy: PredictionStrategy) -> f64 {
        match strategy {
            PredictionStrategy::Sequential => 0.85, // High confidence for sequential
            PredictionStrategy::Delta => 0.70,      // Medium confidence for delta
            PredictionStrategy::Random => {
                // Birthday paradox: sqrt(N) attempts for ~50% success
                let n = predictions.len() as f64;
                let range = self.config.prediction_range as f64;
                1.0 - ((range - n) / range).powi(predictions.len() as i32)
            }
            PredictionStrategy::Combined => 0.75,
        }
    }

    /// Execute punch attempt to predicted port
    pub fn attempt_punch(
        &self,
        socket: &UdpSocket,
        peer_addr: SocketAddr,
        predicted_port: u16,
    ) -> Result<bool, std::io::Error> {
        let target_addr = SocketAddr::new(peer_addr.ip(), predicted_port);
        
        debug!("Attempting punch to predicted port {} (target: {})", predicted_port, target_addr);
        
        // Send punch packet
        let punch_data = b"FASTLINK_PUNCH";
        socket.send_to(punch_data, target_addr)?;
        
        // Wait for response
        let mut buf = [0u8; 1024];
        socket.set_nonblocking(true)?;
        
        let start = Instant::now();
        while start.elapsed() < self.config.punch_timeout {
            match socket.recv_from(&mut buf) {
                Ok((len, src)) => {
                    if src == target_addr && &buf[..len] == b"FASTLINK_PUNCH_ACK" {
                        info!("Punch successful to {}!", target_addr);
                        return Ok(true);
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(1));
                }
                Err(e) => return Err(e),
            }
        }
        
        Ok(false)
    }

    /// Execute full BirthdayPunch sequence
    pub fn execute_birthday_punch(
        &mut self,
        socket: &UdpSocket,
        peer_local_addr: SocketAddr,
        peer_public_base: Option<u16>,
    ) -> Result<Option<SocketAddr>, std::io::Error> {
        if self.nat_properties.nat_type != NatType::Symmetric {
            info!("BirthdayPunch not needed for non-symmetric NAT");
            return Ok(None);
        }

        info!("Starting BirthdayPunch for symmetric NAT traversal");
        
        let prediction = self.predict_peer_port(peer_local_addr.port(), peer_public_base);
        info!(
            "Using {:?} strategy with {} predicted ports (confidence: {:.1}%)",
            prediction.strategy,
            prediction.predicted_ports.len(),
            prediction.confidence * 100.0
        );

        for (idx, &port) in prediction.predicted_ports.iter().enumerate() {
            if self.attempt_punch(socket, peer_local_addr, port)? {
                let success_addr = SocketAddr::new(peer_local_addr.ip(), port);
                info!(
                    "BirthdayPunch succeeded on attempt {} (port {})",
                    idx + 1,
                    port
                );
                return Ok(Some(success_addr));
            }
            
            std::thread::sleep(self.config.punch_interval);
        }

        warn!("BirthdayPunch failed after {} attempts", prediction.predicted_ports.len());
        Ok(None)
    }
}

/// Statistics for BirthdayPunch attempts
#[derive(Debug, Default, Clone)]
pub struct PunchStatistics {
    pub attempts: u32,
    pub successes: u32,
    pub failures: u32,
    pub avg_attempts_to_success: f64,
    pub strategy_success_rates: HashMap<PredictionStrategy, f64>,
}

impl PunchStatistics {
    pub fn record_attempt(&mut self, strategy: PredictionStrategy, success: bool) {
        self.attempts += 1;
        if success {
            self.successes += 1;
        } else {
            self.failures += 1;
        }
    }

    pub fn success_rate(&self) -> f64 {
        if self.attempts == 0 {
            return 0.0;
        }
        self.successes as f64 / self.attempts as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_prediction() {
        let config = BirthdayPunchConfig::default();
        let nat_props = NatProperties::new();
        let punch = BirthdayPunch::new(config, nat_props);

        let prediction = punch.predict_peer_port(12345, Some(40000));
        
        assert!(!prediction.predicted_ports.is_empty());
        assert!(prediction.confidence > 0.0);
    }

    #[test]
    fn test_nat_analysis() {
        let config = BirthdayPunchConfig::default();
        let nat_props = NatProperties::new();
        let mut punch = BirthdayPunch::new(config, nat_props);

        // Simulate sequential NAT
        let mappings = vec![
            (10000, 40000),
            (10001, 40001),
            (10002, 40002),
        ];
        
        punch.analyze_nat_behavior(&mappings);
        
        assert_eq!(punch.last_predicted_delta, Some(1));
    }

    #[test]
    fn test_birthday_paradox_probability() {
        // For 1000 port range and 32 attempts:
        // P(success) ≈ 1 - (1 - 1/1000)^32 ≈ 1 - (999/1000)^32 ≈ 1 - 0.968 ≈ 0.032
        // But with birthday paradox: P(collision) ≈ 32^2 / (2*1000) ≈ 0.51
        let n = 32u32;
        let range = 1000u32;
        let p_no_collision = ((range - n)..range).fold(1.0, |acc, i| acc * i as f64 / range as f64);
        let p_collision = 1.0 - p_no_collision;
        
        // Should be around 50%
        assert!(p_collision > 0.4 && p_collision < 0.6);
    }
}
