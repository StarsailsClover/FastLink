//! FastLink Replay Protection Module
//!
//! Replay attack protection using sliding window and bloom filter

use std::collections::{HashSet, VecDeque};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayWindow {
    window_size: usize,
    seen_messages: HashSet<u64>,
    sequence_numbers: VecDeque<u64>,
}

impl ReplayWindow {
    pub fn new(window_size: usize) -> Self {
        Self {
            window_size,
            seen_messages: HashSet::new(),
            sequence_numbers: VecDeque::new(),
        }
    }

    pub fn is_new(&self, sequence: u64) -> bool {
        !self.seen_messages.contains(&sequence)
    }

    pub fn mark_seen(&mut self, sequence: u64) {
        if self.sequence_numbers.len() >= self.window_size {
            if let Some(old) = self.sequence_numbers.pop_front() {
                self.seen_messages.remove(&old);
            }
        }
        
        self.seen_messages.insert(sequence);
        self.sequence_numbers.push_back(sequence);
    }

    pub fn check_and_mark(&mut self, sequence: u64) -> bool {
        if self.is_new(sequence) {
            self.mark_seen(sequence);
            true
        } else {
            false
        }
    }

    pub fn reset(&mut self) {
        self.seen_messages.clear();
        self.sequence_numbers.clear();
    }
}

pub struct BloomFilterReplay {
    items: usize,
    false_positive_rate: f64,
}

impl BloomFilterReplay {
    pub fn new(items: usize, false_positive_rate: f64) -> Self {
        Self {
            items,
            false_positive_rate,
        }
    }

    pub fn optimal_size(&self) -> usize {
        let m = -(self.items as f64 * self.false_positive_rate.ln()) 
            / (2.0f64.sqrt().powf(2.0).ln());
        m as usize
    }

    pub fn optimal_hashes(&self) -> usize {
        let m = self.optimal_size() as f64;
        let k = (m / self.items as f64) * 2.0f64.ln();
        k.max(1.0) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replay_window() {
        let mut window = ReplayWindow::new(10);
        
        assert!(window.check_and_mark(1));
        assert!(!window.check_and_mark(1));
        assert!(window.check_and_mark(2));
        assert!(!window.check_and_mark(2));
    }

    #[test]
    fn test_replay_window_wrap() {
        let mut window = ReplayWindow::new(3);
        
        window.check_and_mark(1);
        window.check_and_mark(2);
        window.check_and_mark(3);
        
        assert!(window.check_and_mark(4));
        
        assert!(!window.check_and_mark(1));
    }

    #[test]
    fn test_bloom_filter_params() {
        let filter = BloomFilterReplay::new(1000, 0.01);
        let size = filter.optimal_size();
        let hashes = filter.optimal_hashes();
        
        assert!(size > 0);
        assert!(hashes > 0);
    }
}
