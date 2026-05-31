//! FastLink Common Library
//!
//! Shared utilities and types used across all FastLink components

pub mod error;
pub mod serialization;
pub mod logging;
pub mod config;
pub mod platform;
pub mod time;

pub use error::{CommonError, Result};
pub use serialization::{serialize, deserialize, to_json, from_json};
pub use logging::{init_logging, init_logging_with_filter, init_json_logging};
pub use config::Config;
pub use platform::{is_windows, is_linux, is_macos};
pub use time::Timestamp;
pub use std::time::{Duration, Instant};
