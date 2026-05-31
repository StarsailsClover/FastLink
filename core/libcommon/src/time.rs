//! FastLink Time Utilities

use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

/// Timestamp wrapper for consistent time handling
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timestamp(SystemTime);

impl Timestamp {
    /// Create a new timestamp from SystemTime
    pub fn new(time: SystemTime) -> Self {
        Self(time)
    }
    
    /// Get current timestamp
    pub fn now() -> Self {
        Self(SystemTime::now())
    }
    
    /// Get duration since UNIX_EPOCH
    pub fn duration_since_epoch(&self) -> Duration {
        self.0.duration_since(UNIX_EPOCH).unwrap_or_default()
    }
    
    /// Get duration since another timestamp
    pub fn duration_since(&self, earlier: Timestamp) -> Duration {
        self.0.duration_since(earlier.0).unwrap_or_default()
    }
    
    /// Add duration to this timestamp
    pub fn checked_add(&self, duration: Duration) -> Option<Timestamp> {
        self.0.checked_add(duration).map(Self)
    }
    
    /// Subtract duration from this timestamp
    pub fn checked_sub(&self, duration: Duration) -> Option<Timestamp> {
        self.0.checked_sub(duration).map(Self)
    }
    
    /// Check if this timestamp is before another
    pub fn is_before(&self, other: Timestamp) -> bool {
        self < &other
    }
    
    /// Check if this timestamp is after another
    pub fn is_after(&self, other: Timestamp) -> bool {
        self > &other
    }
}

impl std::ops::Add<Duration> for Timestamp {
    type Output = Timestamp;
    
    fn add(self, duration: Duration) -> Self::Output {
        Timestamp(self.0 + duration)
    }
}

impl std::ops::Sub<Duration> for Timestamp {
    type Output = Timestamp;
    
    fn sub(self, duration: Duration) -> Self::Output {
        Timestamp(self.0 - duration)
    }
}

impl std::ops::Sub<Timestamp> for Timestamp {
    type Output = Duration;
    
    fn sub(self, other: Timestamp) -> Self::Output {
        self.0.duration_since(other.0).unwrap_or_default()
    }
}

impl Serialize for Timestamp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Serialize as duration since epoch
        let duration = self.duration_since_epoch();
        let secs = duration.as_secs();
        let nanos = duration.subsec_nanos();
        (secs, nanos).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Timestamp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let (secs, nanos): (u64, u32) = Deserialize::deserialize(deserializer)?;
        let duration = Duration::new(secs, nanos);
        let system_time = UNIX_EPOCH + duration;
        Ok(Timestamp(system_time))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_timestamp_operations() {
        let t1 = Timestamp::now();
        let t2 = t1 + Duration::from_secs(1);
        
        assert!(t2.is_after(t1));
        assert_eq!(t2 - t1, Duration::from_secs(1));
    }
    
    #[test]
    fn test_timestamp_serialization() {
        let t = Timestamp::new(UNIX_EPOCH + Duration::from_secs(1000));
        let json = serde_json::to_string(&t).unwrap();
        let decoded: Timestamp = serde_json::from_str(&json).unwrap();
        assert_eq!(t, decoded);
    }
}
