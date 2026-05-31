//! FastLink Test Assertions Module
//!
//! Custom assertions for test scenarios

use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub struct ConnectionStats {
    pub packets_sent: u64,
    pub packets_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub latency_ms: f64,
}

impl ConnectionStats {
    pub fn new() -> Self {
        Self {
            packets_sent: 0,
            packets_received: 0,
            bytes_sent: 0,
            bytes_received: 0,
            latency_ms: 0.0,
        }
    }
    
    pub fn assert_latency_under(&self, max_ms: f64) -> Result<(), AssertionError> {
        if self.latency_ms > max_ms {
            Err(AssertionError::LatencyExceeded {
                actual: self.latency_ms,
                max: max_ms,
            })
        } else {
            Ok(())
        }
    }
    
    pub fn assert_throughput_above(&self, min_mbps: f64) -> Result<(), AssertionError> {
        let throughput = (self.bytes_sent as f64 * 8.0) / 1_000_000.0;
        if throughput < min_mbps {
            Err(AssertionError::ThroughputTooLow {
                actual: throughput,
                min: min_mbps,
            })
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone)]
pub struct MessageAssertion {
    pub sequence: u64,
    pub timestamp: u64,
    pub source: SocketAddr,
    pub destination: SocketAddr,
    pub size: usize,
}

impl MessageAssertion {
    pub fn new(sequence: u64, source: SocketAddr, destination: SocketAddr, size: usize) -> Self {
        Self {
            sequence,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            source,
            destination,
            size,
        }
    }
    
    pub fn assert_sequence_order(&self, previous: &MessageAssertion) -> Result<(), AssertionError> {
        if self.sequence <= previous.sequence {
            Err(AssertionError::SequenceOutOfOrder {
                current: self.sequence,
                previous: previous.sequence,
            })
        } else {
            Ok(())
        }
    }
    
    pub fn assert_timing_within(&self, previous: &MessageAssertion, max_delta_ms: u64) -> Result<(), AssertionError> {
        let delta = self.timestamp.saturating_sub(previous.timestamp);
        if delta > max_delta_ms {
            Err(AssertionError::TimingExceeded {
                actual_delta: delta,
                max_delta: max_delta_ms,
            })
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone)]
pub enum AssertionError {
    LatencyExceeded { actual: f64, max: f64 },
    ThroughputTooLow { actual: f64, min: f64 },
    SequenceOutOfOrder { current: u64, previous: u64 },
    TimingExceeded { actual_delta: u64, max_delta: u64 },
}

impl std::fmt::Display for AssertionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssertionError::LatencyExceeded { actual, max } => {
                write!(f, "Latency {}ms exceeded maximum {}ms", actual, max)
            }
            AssertionError::ThroughputTooLow { actual, min } => {
                write!(f, "Throughput {}Mbps below minimum {}Mbps", actual, min)
            }
            AssertionError::SequenceOutOfOrder { current, previous } => {
                write!(f, "Sequence {} is not greater than previous {}", current, previous)
            }
            AssertionError::TimingExceeded { actual_delta, max_delta } => {
                write!(f, "Timing delta {}ms exceeded maximum {}ms", actual_delta, max_delta)
            }
        }
    }
}

impl std::error::Error for AssertionError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_stats_latency() {
        let stats = ConnectionStats {
            latency_ms: 50.0,
            ..ConnectionStats::new()
        };
        
        assert!(stats.assert_latency_under(100.0).is_ok());
        assert!(stats.assert_latency_under(40.0).is_err());
    }

    #[test]
    fn test_message_sequence_order() {
        let msg1 = MessageAssertion::new(
            1,
            "127.0.0.1:8080".parse().unwrap(),
            "127.0.0.1:8081".parse().unwrap(),
            100,
        );
        
        let msg2 = MessageAssertion::new(
            2,
            "127.0.0.1:8080".parse().unwrap(),
            "127.0.0.1:8081".parse().unwrap(),
            100,
        );
        
        assert!(msg2.assert_sequence_order(&msg1).is_ok());
        assert!(msg1.assert_sequence_order(&msg2).is_err());
    }
}
