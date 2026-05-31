//! FastLink-P2P
//!
//! FastLink peer-to-peer protocol implementation for decentralized communication
//!
//! Features:
//! - Distributed peer discovery and DHT
//! - P2P node management and connection handling
//! - Message routing and peer discovery
//! - NAT traversal support via ICE/STUN
//! - Reliable and unreliable message delivery

pub mod node;
pub mod dht;
pub mod routing;
pub mod connection;
pub mod discovery;

pub use node::*;
pub use dht::*;
pub use routing::*;
pub use connection::*;
pub use discovery::*;
