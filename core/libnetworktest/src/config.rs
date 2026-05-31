//! Configuration for network testing

use crate::models::NetworkScenario;

/// Test configuration
#[derive(Debug, Clone)]
pub struct TestConfig {
    /// Test name
    pub name: String,
    /// Test description
    pub description: String,
    /// Network scenario to use
    pub scenario: NetworkScenario,
    /// Duration of the test
    pub duration_ms: u64,
    /// Number of messages to send
    pub message_count: u64,
    /// Message size (bytes)
    pub message_size: usize,
    /// Number of concurrent connections
    pub concurrency: usize,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            name: "Default Test".to_string(),
            description: "Default network test configuration".to_string(),
            scenario: NetworkScenario::HomeBroadband,
            duration_ms: 60000,
            message_count: 1000,
            message_size: 1024,
            concurrency: 1,
        }
    }
}

/// Test suite configuration
#[derive(Debug, Clone)]
pub struct TestSuite {
    /// Individual test configurations
    pub tests: Vec<TestConfig>,
    /// Output directory
    pub output_dir: Option<String>,
    /// Whether to generate visual report
    pub generate_report: bool,
}

impl TestSuite {
    /// Create a new test suite
    pub fn new() -> Self {
        Self {
            tests: Vec::new(),
            output_dir: None,
            generate_report: false,
        }
    }
    
    /// Add a test to the suite
    pub fn add_test(&mut self, test: TestConfig) {
        self.tests.push(test);
    }
    
    /// Create a quick benchmark suite
    pub fn benchmark_suite() -> Self {
        let mut suite = Self::new();
        
        suite.add_test(TestConfig {
            name: "Perfect Network".to_string(),
            description: "Test under perfect network conditions".to_string(),
            scenario: NetworkScenario::Perfect,
            duration_ms: 30000,
            message_count: 10000,
            message_size: 1024,
            concurrency: 10,
        });
        
        suite.add_test(TestConfig {
            name: "Home Broadband".to_string(),
            description: "Test under typical home broadband conditions".to_string(),
            scenario: NetworkScenario::HomeBroadband,
            duration_ms: 30000,
            message_count: 5000,
            message_size: 1024,
            concurrency: 10,
        });
        
        suite.add_test(TestConfig {
            name: "Mobile 3G".to_string(),
            description: "Test under 3G mobile network conditions".to_string(),
            scenario: NetworkScenario::Mobile3G,
            duration_ms: 60000,
            message_count: 1000,
            message_size: 512,
            concurrency: 5,
        });
        
        suite.add_test(TestConfig {
            name: "Mobile 4G".to_string(),
            description: "Test under 4G mobile network conditions".to_string(),
            scenario: NetworkScenario::Mobile4G,
            duration_ms: 30000,
            message_count: 3000,
            message_size: 1024,
            concurrency: 10,
        });
        
        suite.add_test(TestConfig {
            name: "Mobile 5G".to_string(),
            description: "Test under 5G mobile network conditions".to_string(),
            scenario: NetworkScenario::Mobile5G,
            duration_ms: 30000,
            message_count: 8000,
            message_size: 1024,
            concurrency: 15,
        });
        
        suite.add_test(TestConfig {
            name: "Satellite".to_string(),
            description: "Test under satellite network conditions".to_string(),
            scenario: NetworkScenario::Satellite,
            duration_ms: 60000,
            message_count: 500,
            message_size: 512,
            concurrency: 5,
        });
        
        suite.add_test(TestConfig {
            name: "Rural Slow".to_string(),
            description: "Test under slow rural network conditions".to_string(),
            scenario: NetworkScenario::RuralSlow,
            duration_ms: 90000,
            message_count: 200,
            message_size: 256,
            concurrency: 3,
        });
        
        suite
    }
}

impl Default for TestSuite {
    fn default() -> Self {
        Self::new()
    }
}
