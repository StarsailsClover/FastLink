//! Test framework for FastLink
//! 
//! Provides test utilities, network simulation, and test scenarios

pub mod network;
pub mod scenarios;
pub mod assertions;

pub use network::{SimulatedNetwork, NetworkCondition};
pub use scenarios::{TestScenario, ScenarioRunner};
