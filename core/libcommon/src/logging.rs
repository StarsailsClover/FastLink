//! FastLink Logging Utilities

use tracing_subscriber::{layer::SubscriberExt, EnvFilter};

/// Initialize logging with default configuration
pub fn init_logging() {
    let filter = EnvFilter::from_default_env()
        .add_directive(tracing::Level::INFO.into());
    
    let subscriber = tracing_subscriber::registry()
        .with(filter);
    
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set global subscriber");
}

/// Initialize logging with custom filter
pub fn init_logging_with_filter(filter: impl Into<String>) {
    let filter = EnvFilter::new(filter.into());
    
    let subscriber = tracing_subscriber::registry()
        .with(filter);
    
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set global subscriber");
}

/// Initialize JSON logging
pub fn init_json_logging() {
    let filter = EnvFilter::from_default_env()
        .add_directive(tracing::Level::INFO.into());
    
    let subscriber = tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer().json());
    
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set global subscriber");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_init_logging() {
        init_logging_with_filter("debug");
        tracing::info!("Test log message");
    }
}
