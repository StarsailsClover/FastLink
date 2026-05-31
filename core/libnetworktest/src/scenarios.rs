//! Predefined test scenarios and utilities

use std::time::Duration;
use bytes::Bytes;

use crate::models::{Packet, NetworkScenario};
use crate::emulator::NetworkEmulator;
use crate::metrics::{NetworkMetrics, MetricsSummary};
use crate::config::{TestConfig, TestSuite};

/// Test result from a single test
#[derive(Debug, Clone)]
pub struct TestResult {
    pub config: TestConfig,
    pub metrics: MetricsSummary,
    pub duration: Duration,
    pub success: bool,
    pub error_message: Option<String>,
}

/// Test runner that executes test suites
pub struct TestRunner {
    current_metrics: NetworkMetrics,
}

impl TestRunner {
    /// Create a new test runner
    pub fn new() -> Self {
        Self {
            current_metrics: NetworkMetrics::new(),
        }
    }
    
    /// Run a single test configuration
    pub async fn run_test(&mut self, config: TestConfig) -> TestResult {
        println!("Running test: {}", config.name);
        println!("Description: {}", config.description);
        
        let start_time = std::time::Instant::now();
        let conditions = config.scenario.conditions();
        
        // Create network emulator
        let (_, incoming_tx, mut outgoing_rx) = NetworkEmulator::new(conditions);
        
        // Generate and send test packets
        let mut packets_sent = 0;
        let mut packet_id = 0;
        
        // Start receiver task
        let mut received_count = 0;
        let receiver = tokio::spawn(async move {
            while let Some(_) = outgoing_rx.recv().await {
                received_count += 1;
            }
            received_count
        });
        
        // Send packets
        let test_data: Vec<u8> = vec![0u8; config.message_size];
        let src_addr: std::net::SocketAddr = "127.0.0.1:10000".parse().unwrap();
        let dst_addr: std::net::SocketAddr = "127.0.0.1:10001".parse().unwrap();
        
        let send_future = async {
            for _ in 0..config.message_count {
                let packet = Packet {
                    data: Bytes::from(test_data.clone()),
                    src: src_addr,
                    dst: dst_addr,
                    sent_at: Duration::from_secs(0),
                    id: packet_id,
                };
                
                if incoming_tx.send(packet).await.is_err() {
                    break;
                }
                
                packets_sent += 1;
                packet_id += 1;
                
                // Small delay to control throughput
                tokio::time::sleep(Duration::from_micros(10)).await;
            }
        };
        
        // Run for duration
        let _result = tokio::time::timeout(
            Duration::from_millis(config.duration_ms),
            send_future,
        ).await;
        
        // Get receiver result
        drop(incoming_tx);
        let received = receiver.await.unwrap_or(0);
        
        let duration = start_time.elapsed();
        
        // Create metrics
        let mut metrics = NetworkMetrics::new();
        metrics.total_sent = packets_sent;
        metrics.total_received = received;
        metrics.loss_rate = if packets_sent > 0 {
            (packets_sent - received) as f64 / packets_sent as f64
        } else {
            0.0
        };
        
        println!("Test completed: {}", config.name);
        println!("Packets sent: {}", packets_sent);
        println!("Packets received: {}", received);
        println!("Duration: {:?}", duration);
        
        TestResult {
            config,
            metrics: metrics.summary(),
            duration,
            success: true,
            error_message: None,
        }
    }
    
    /// Run an entire test suite
    pub async fn run_suite(&mut self, suite: TestSuite) -> Vec<TestResult> {
        let mut results = Vec::new();
        let test_count = suite.tests.len();
        
        println!("Starting test suite with {} tests", test_count);
        
        for (idx, test) in suite.tests.into_iter().enumerate() {
            println!("\n=== Test {}/{} ===", idx + 1, test_count);
            
            let result = self.run_test(test).await;
            results.push(result);
        }
        
        println!("\n=== Test Suite Complete ===");
        
        results
    }
    
    /// Print test results summary
    pub fn print_results(results: &[TestResult]) {
        println!("\n=== Test Results Summary ===");
        
        for result in results {
            println!("\nTest: {}", result.config.name);
            println!("Duration: {:?}", result.duration);
            println!("Success: {}", result.success);
            
            if let Some(msg) = &result.error_message {
                println!("Error: {}", msg);
            }
            
            result.metrics.print();
        }
        
        println!("\n=========================");
    }
}

impl Default for TestRunner {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to run a quick test
pub async fn quick_test(
    scenario: NetworkScenario,
    message_count: u64,
) -> TestResult {
    let config = TestConfig {
        name: "Quick Test".to_string(),
        description: format!("Quick test with {:?} scenario", scenario),
        scenario,
        duration_ms: 30000,
        message_count,
        message_size: 1024,
        concurrency: 1,
    };
    
    let mut runner = TestRunner::new();
    runner.run_test(config).await
}
