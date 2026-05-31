//! FastLink-Games Protocol
//!
//! A low-latency, real-time protocol optimized for multiplayer gaming.
//! Provides state synchronization, player input streaming, and event broadcasting.

use std::collections::{HashMap, VecDeque};
use std::net::SocketAddr;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, info, warn};
use serde::{Deserialize, Serialize};

/// Games protocol error type
#[derive(Debug, Error)]
pub enum GamesError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Connection closed")]
    ConnectionClosed,
    #[error("Invalid state")]
    InvalidState,
    #[error("Room not found")]
    RoomNotFound,
    #[error("Player not found")]
    PlayerNotFound,
}

/// Game room state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameRoomState {
    pub room_id: String,
    pub players: HashMap<String, PlayerState>,
    pub game_state: Vec<u8>,
    pub tick: u64,
}

/// Player state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {
    pub player_id: String,
    pub position: (f32, f32, f32),
    pub rotation: (f32, f32, f32, f32),
    pub velocity: (f32, f32, f32),
    pub input: PlayerInput,
    pub health: f32,
    pub is_connected: bool,
}

/// Player input
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlayerInput {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub jump: bool,
    pub shoot: bool,
    pub mouse_delta: (f32, f32),
    pub timestamp: u64,
}

/// Game message type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameMessage {
    /// Join a game room
    JoinRoom {
        room_id: String,
        player_id: String,
    },
    /// Leave a game room
    LeaveRoom {
        room_id: String,
        player_id: String,
    },
    /// Player input
    PlayerInput {
        player_id: String,
        input: PlayerInput,
        tick: u64,
    },
    /// State update (server to client)
    StateUpdate {
        state: GameRoomState,
    },
    /// Game event
    GameEvent {
        event_type: String,
        data: Vec<u8>,
    },
    /// Ping
    Ping {
        timestamp: u64,
    },
    /// Pong
    Pong {
        timestamp: u64,
    },
    /// Acknowledgement
    Ack {
        tick: u64,
    },
}

/// Room configuration
#[derive(Debug, Clone)]
pub struct RoomConfig {
    pub max_players: usize,
    pub tick_rate: u32,
    pub state_buffer_size: usize,
}

impl Default for RoomConfig {
    fn default() -> Self {
        Self {
            max_players: 16,
            tick_rate: 60,
            state_buffer_size: 120,
        }
    }
}

/// Game room
pub struct GameRoom {
    pub room_id: String,
    config: RoomConfig,
    players: Arc<RwLock<HashMap<String, PlayerConnection>>>,
    state_history: VecDeque<GameRoomState>,
    current_tick: u64,
}

impl GameRoom {
    /// Create a new game room
    pub fn new(room_id: String, config: RoomConfig) -> Self {
        Self {
            room_id,
            config,
            players: Arc::new(RwLock::new(HashMap::new())),
            state_history: VecDeque::new(),
            current_tick: 0,
        }
    }

    /// Add a player to the room
    pub async fn add_player(&mut self, player_id: String, addr: SocketAddr) -> Result<(), GamesError> {
        let mut players = self.players.write().await;
        
        if players.len() >= self.config.max_players {
            return Err(GamesError::InvalidState);
        }
        
        let connection = PlayerConnection {
            player_id: player_id.clone(),
            addr,
            ping: 0,
            last_seen: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        
        players.insert(player_id, connection);
        Ok(())
    }

    /// Remove a player from the room
    pub async fn remove_player(&mut self, player_id: &str) {
        self.players.write().await.remove(player_id);
    }

    /// Get current player count
    pub async fn player_count(&self) -> usize {
        self.players.read().await.len()
    }

    /// Process player input
    pub async fn process_input(&mut self, player_id: &str, input: PlayerInput, tick: u64) {
        // TODO: Implement input processing and state update
        debug!("Processing input from {} at tick {}", player_id, tick);
    }

    /// Get current state
    pub fn current_state(&self) -> Option<&GameRoomState> {
        self.state_history.back()
    }
}

/// Player connection
#[derive(Debug, Clone)]
pub struct PlayerConnection {
    pub player_id: String,
    pub addr: SocketAddr,
    pub ping: u64,
    pub last_seen: u64,
}

/// Game server
pub struct GameServer {
    rooms: Arc<RwLock<HashMap<String, GameRoom>>>,
    default_room_config: RoomConfig,
}

impl GameServer {
    /// Create a new game server
    pub fn new(default_config: RoomConfig) -> Self {
        Self {
            rooms: Arc::new(RwLock::new(HashMap::new())),
            default_room_config: default_config,
        }
    }

    /// Create a new game room
    pub async fn create_room(&self, room_id: String) -> Result<(), GamesError> {
        let mut rooms = self.rooms.write().await;
        
        if rooms.contains_key(&room_id) {
            return Err(GamesError::InvalidState);
        }
        
        let room = GameRoom::new(room_id.clone(), self.default_room_config.clone());
        rooms.insert(room_id, room);
        
        Ok(())
    }

    /// Get a game room
    pub async fn get_room(&self, room_id: &str) -> Option<GameRoom> {
        // We can't return a reference across async, so we'd need to redesign
        None
    }

    /// List all active rooms
    pub async fn list_rooms(&self) -> Vec<String> {
        self.rooms.read().await.keys().cloned().collect()
    }
}

/// Game client
pub struct GameClient {
    player_id: String,
    current_room: Option<String>,
    input_buffer: VecDeque<(u64, PlayerInput)>,
    state_buffer: VecDeque<GameRoomState>,
    local_tick: u64,
}

impl GameClient {
    /// Create a new game client
    pub fn new(player_id: String) -> Self {
        Self {
            player_id,
            current_room: None,
            input_buffer: VecDeque::new(),
            state_buffer: VecDeque::new(),
            local_tick: 0,
        }
    }

    /// Send player input
    pub fn send_input(&mut self, input: PlayerInput) {
        self.input_buffer.push_back((self.local_tick, input));
        self.local_tick += 1;
    }

    /// Receive state update
    pub fn receive_state(&mut self, state: GameRoomState) {
        self.state_buffer.push_back(state);
        
        // Keep only the last N states
        while self.state_buffer.len() > 120 {
            self.state_buffer.pop_front();
        }
    }

    /// Interpolate between states for smooth rendering
    pub fn interpolate_state(&self, _t: f32) -> Option<GameRoomState> {
        // TODO: Implement state interpolation
        None
    }

    /// Extrapolate current state from past states
    pub fn extrapolate_state(&self) -> Option<GameRoomState> {
        // TODO: Implement state extrapolation (client-side prediction)
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_room_config_default() {
        let config = RoomConfig::default();
        assert_eq!(config.max_players, 16);
        assert_eq!(config.tick_rate, 60);
    }

    #[test]
    fn test_player_input_default() {
        let input = PlayerInput::default();
        assert!(!input.forward);
        assert!(!input.jump);
    }

    #[test]
    fn test_game_client_creation() {
        let client = GameClient::new("player1".to_string());
        assert_eq!(client.player_id, "player1");
    }
}
