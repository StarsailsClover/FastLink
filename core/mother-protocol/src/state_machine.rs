//! FastLink State Machine
//!
//! Connection state machine implementation

use std::sync::Arc;
use parking_lot::RwLock;
use std::time::{Duration, Instant};
use crate::traits::ConnectionState;
use crate::error::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateEvent {
    Connect,
    HandshakeStart,
    HandshakeComplete,
    DataReceived,
    DataSent,
    Close,
    Timeout,
    Error,
}

pub struct ConnectionStateMachine {
    state: RwLock<ConnectionState>,
    last_event: RwLock<Instant>,
    state_history: RwLock<Vec<(Instant, ConnectionState)>>,
}

impl ConnectionStateMachine {
    pub fn new() -> Self {
        Self {
            state: RwLock::new(ConnectionState::Closed),
            last_event: RwLock::new(Instant::now()),
            state_history: RwLock::new(Vec::new()),
        }
    }
    
    pub fn current_state(&self) -> ConnectionState {
        *self.state.read()
    }
    
    pub fn transition(&mut self, event: StateEvent) -> Result<(), Error> {
        let current = self.current_state();
        let new_state = self.next_state(current, event)?;
        
        let now = Instant::now();
        *self.state.write() = new_state;
        *self.last_event.write() = now;
        
        self.state_history.write().push((now, new_state));
        
        Ok(())
    }
    
    fn next_state(&self, current: ConnectionState, event: StateEvent) -> Result<ConnectionState, Error> {
        match (current, event) {
            (ConnectionState::Closed, StateEvent::Connect) => Ok(ConnectionState::Connecting),
            (ConnectionState::Connecting, StateEvent::HandshakeStart) => Ok(ConnectionState::Handshake),
            (ConnectionState::Handshake, StateEvent::HandshakeComplete) => Ok(ConnectionState::Established),
            (ConnectionState::Established, StateEvent::DataReceived | StateEvent::DataSent) => Ok(ConnectionState::Established),
            (ConnectionState::Established, StateEvent::Close) => Ok(ConnectionState::Closing),
            (ConnectionState::Closing, StateEvent::Close) => Ok(ConnectionState::Closed),
            (_, StateEvent::Timeout | StateEvent::Error) => Ok(ConnectionState::Closed),
            _ => Err(Error::InvalidFormat),
        }
    }
    
    pub fn last_event_time(&self) -> Instant {
        *self.last_event.read()
    }
    
    pub fn time_in_state(&self) -> Duration {
        let last = self.last_event_time();
        last.elapsed()
    }
    
    pub fn is_established(&self) -> bool {
        self.current_state() == ConnectionState::Established
    }
    
    pub fn is_closed(&self) -> bool {
        matches!(
            self.current_state(),
            ConnectionState::Closed | ConnectionState::Closing
        )
    }
    
    pub fn reset(&mut self) {
        *self.state.write() = ConnectionState::Closed;
        *self.last_event.write() = Instant::now();
    }
}

impl Default for ConnectionStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let sm = ConnectionStateMachine::new();
        assert_eq!(sm.current_state(), ConnectionState::Closed);
    }

    #[test]
    fn test_state_transitions() {
        let mut sm = ConnectionStateMachine::new();
        
        assert!(sm.transition(StateEvent::Connect).is_ok());
        assert_eq!(sm.current_state(), ConnectionState::Connecting);
        
        assert!(sm.transition(StateEvent::HandshakeStart).is_ok());
        assert_eq!(sm.current_state(), ConnectionState::Handshake);
        
        assert!(sm.transition(StateEvent::HandshakeComplete).is_ok());
        assert_eq!(sm.current_state(), ConnectionState::Established);
    }

    #[test]
    fn test_close_transition() {
        let mut sm = ConnectionStateMachine::new();
        
        sm.transition(StateEvent::Connect).unwrap();
        sm.transition(StateEvent::HandshakeStart).unwrap();
        sm.transition(StateEvent::HandshakeComplete).unwrap();
        
        assert!(sm.transition(StateEvent::Close).is_ok());
        assert_eq!(sm.current_state(), ConnectionState::Closing);
        
        assert!(sm.transition(StateEvent::Close).is_ok());
        assert_eq!(sm.current_state(), ConnectionState::Closed);
    }

    #[test]
    fn test_is_established() {
        let mut sm = ConnectionStateMachine::new();
        
        assert!(!sm.is_established());
        
        sm.transition(StateEvent::Connect).unwrap();
        sm.transition(StateEvent::HandshakeStart).unwrap();
        sm.transition(StateEvent::HandshakeComplete).unwrap();
        
        assert!(sm.is_established());
    }
}
