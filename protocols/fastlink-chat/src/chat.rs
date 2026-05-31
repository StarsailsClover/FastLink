//! FastLink-Chat Protocol
//!
//! An instant messaging protocol with support for private chats, group chats,
//! file transfers, and message history.

use std::collections::{HashMap, VecDeque};
use std::net::SocketAddr;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, info, warn};
use serde::{Deserialize, Serialize};

/// Chat protocol error type
#[derive(Debug, Error)]
pub enum ChatError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("User not found")]
    UserNotFound,
    #[error("Room not found")]
    RoomNotFound,
    #[error("Not authorized")]
    NotAuthorized,
    #[error("Connection closed")]
    ConnectionClosed,
    #[error("Message too large")]
    MessageTooLarge,
}

/// Chat user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatUser {
    pub user_id: String,
    pub username: String,
    pub display_name: String,
    pub avatar: Option<Vec<u8>>,
    pub is_online: bool,
    pub last_seen: u64,
}

/// Chat room
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRoom {
    pub room_id: String,
    pub name: String,
    pub description: Option<String>,
    pub members: Vec<String>,
    pub is_private: bool,
    pub created_at: u64,
}

/// Chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub message_id: String,
    pub room_id: String,
    pub sender_id: String,
    pub sender_name: String,
    pub content: MessageContent,
    pub timestamp: u64,
    pub is_read: bool,
    pub reply_to: Option<String>,
}

/// Message content type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageContent {
    /// Text message
    Text(String),
    /// Image
    Image {
        data: Vec<u8>,
        format: String,
    },
    /// File
    File {
        name: String,
        data: Vec<u8>,
        size: u64,
    },
    /// Reaction
    Reaction {
        emoji: String,
        to_message_id: String,
    },
    /// System message
    System(String),
}

/// Chat message type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChatProtocolMessage {
    /// Send a chat message
    SendMessage {
        room_id: String,
        content: MessageContent,
        reply_to: Option<String>,
    },
    /// Receive a chat message
    ReceiveMessage {
        message: ChatMessage,
    },
    /// Join a chat room
    JoinRoom {
        room_id: String,
    },
    /// Leave a chat room
    LeaveRoom {
        room_id: String,
    },
    /// Create a chat room
    CreateRoom {
        name: String,
        description: Option<String>,
        is_private: bool,
        initial_members: Vec<String>,
    },
    /// Room created response
    RoomCreated {
        room: ChatRoom,
    },
    /// Request message history
    RequestHistory {
        room_id: String,
        limit: u32,
        before_timestamp: Option<u64>,
    },
    /// Message history response
    HistoryResponse {
        room_id: String,
        messages: Vec<ChatMessage>,
    },
    /// Update presence
    UpdatePresence {
        user: ChatUser,
    },
    /// Typing indicator
    Typing {
        room_id: String,
        user_id: String,
        is_typing: bool,
    },
    /// Mark messages as read
    MarkRead {
        room_id: String,
        message_ids: Vec<String>,
    },
}

/// Chat server configuration
#[derive(Debug, Clone)]
pub struct ChatServerConfig {
    pub max_message_size: usize,
    pub max_history_size: usize,
    pub enable_file_transfer: bool,
    pub max_file_size: u64,
}

impl Default for ChatServerConfig {
    fn default() -> Self {
        Self {
            max_message_size: 64 * 1024, // 64 KB
            max_history_size: 1000,
            enable_file_transfer: true,
            max_file_size: 100 * 1024 * 1024, // 100 MB
        }
    }
}

/// Chat server
pub struct ChatServer {
    config: ChatServerConfig,
    users: Arc<RwLock<HashMap<String, ChatUser>>>,
    rooms: Arc<RwLock<HashMap<String, ChatRoom>>>,
    message_history: Arc<RwLock<HashMap<String, VecDeque<ChatMessage>>>>,
    connections: Arc<RwLock<HashMap<String, mpsc::Sender<ChatProtocolMessage>>>>,
}

impl ChatServer {
    /// Create a new chat server
    pub fn new(config: ChatServerConfig) -> Self {
        Self {
            config,
            users: Arc::new(RwLock::new(HashMap::new())),
            rooms: Arc::new(RwLock::new(HashMap::new())),
            message_history: Arc::new(RwLock::new(HashMap::new())),
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new user
    pub async fn register_user(&self, user_id: String, username: String, display_name: String) -> Result<(), ChatError> {
        let mut users = self.users.write().await;
        
        let user = ChatUser {
            user_id: user_id.clone(),
            username,
            display_name,
            avatar: None,
            is_online: false,
            last_seen: 0,
        };
        
        users.insert(user_id, user);
        Ok(())
    }

    /// Connect a user
    pub async fn connect_user(&self, user_id: String, sender: mpsc::Sender<ChatProtocolMessage>) -> Result<(), ChatError> {
        let mut users = self.users.write().await;
        
        if let Some(user) = users.get_mut(&user_id) {
            user.is_online = true;
            user.last_seen = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
        } else {
            return Err(ChatError::UserNotFound);
        }
        
        let mut connections = self.connections.write().await;
        connections.insert(user_id, sender);
        Ok(())
    }

    /// Disconnect a user
    pub async fn disconnect_user(&self, user_id: &str) {
        let mut users = self.users.write().await;
        
        if let Some(user) = users.get_mut(user_id) {
            user.is_online = false;
            user.last_seen = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
        }
        
        let mut connections = self.connections.write().await;
        connections.remove(user_id);
    }

    /// Create a new chat room
    pub async fn create_room(&self, creator_id: String, name: String, description: Option<String>, is_private: bool, initial_members: Vec<String>) -> Result<ChatRoom, ChatError> {
        let mut rooms = self.rooms.write().await;
        
        let room_id = format!("room_{}", uuid::Uuid::new_v4().simple());
        
        let mut members = initial_members.clone();
        if !members.contains(&creator_id) {
            members.push(creator_id);
        }
        
        let room = ChatRoom {
            room_id: room_id.clone(),
            name,
            description,
            members,
            is_private,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        
        rooms.insert(room_id.clone(), room.clone());
        
        // Initialize message history for this room
        let mut history = self.message_history.write().await;
        history.insert(room_id.clone(), VecDeque::new());
        
        Ok(room)
    }

    /// Send a message to a room
    pub async fn send_message(&self, sender_id: String, room_id: String, content: MessageContent, reply_to: Option<String>) -> Result<ChatMessage, ChatError> {
        // Check room exists and user is a member
        let rooms = self.rooms.read().await;
        let room = rooms.get(&room_id).ok_or(ChatError::RoomNotFound)?;
        
        if !room.members.contains(&sender_id) {
            return Err(ChatError::NotAuthorized);
        }
        
        // Get sender info
        let users = self.users.read().await;
        let sender = users.get(&sender_id).ok_or(ChatError::UserNotFound)?;
        
        let message_id = format!("msg_{}", uuid::Uuid::new_v4().simple());
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let message = ChatMessage {
            message_id: message_id.clone(),
            room_id: room_id.clone(),
            sender_id: sender_id.clone(),
            sender_name: sender.display_name.clone(),
            content,
            timestamp,
            is_read: false,
            reply_to,
        };
        
        // Add to history
        let mut history = self.message_history.write().await;
        if let Some(room_history) = history.get_mut(&room_id) {
            room_history.push_back(message.clone());
            
            // Keep only last N messages
            while room_history.len() > self.config.max_history_size {
                room_history.pop_front();
            }
        }
        
        // Broadcast to all room members
        drop(history);
        drop(users);
        drop(rooms);
        
        self.broadcast_to_room(&room_id, ChatProtocolMessage::ReceiveMessage { message: message.clone() }).await;
        
        Ok(message)
    }

    /// Broadcast a message to all room members
    async fn broadcast_to_room(&self, room_id: &str, message: ChatProtocolMessage) {
        let rooms = self.rooms.read().await;
        let Some(room) = rooms.get(room_id) else {
            return;
        };
        
        let connections = self.connections.read().await;
        
        for member_id in &room.members {
            if let Some(sender) = connections.get(member_id) {
                let _ = sender.send(message.clone()).await;
            }
        }
    }

    /// Get message history for a room
    pub async fn get_history(&self, room_id: String, limit: u32, before_timestamp: Option<u64>) -> Result<Vec<ChatMessage>, ChatError> {
        let history = self.message_history.read().await;
        let room_history = history.get(&room_id).ok_or(ChatError::RoomNotFound)?;
        
        let messages: Vec<ChatMessage> = room_history
            .iter()
            .filter(|msg| {
                if let Some(ts) = before_timestamp {
                    msg.timestamp < ts
                } else {
                    true
                }
            })
            .rev()
            .take(limit as usize)
            .cloned()
            .collect();
        
        Ok(messages)
    }

    /// Get online users in a room
    pub async fn get_room_members(&self, room_id: &str) -> Result<Vec<ChatUser>, ChatError> {
        let rooms = self.rooms.read().await;
        let room = rooms.get(room_id).ok_or(ChatError::RoomNotFound)?;
        
        let users = self.users.read().await;
        
        let members: Vec<ChatUser> = room.members
            .iter()
            .filter_map(|user_id| users.get(user_id).cloned())
            .collect();
        
        Ok(members)
    }
}

/// Chat client
pub struct ChatClient {
    user_id: String,
    current_room: Option<String>,
    rooms: Vec<ChatRoom>,
    message_cache: HashMap<String, VecDeque<ChatMessage>>,
    server_sender: Option<mpsc::Sender<ChatProtocolMessage>>,
}

impl ChatClient {
    /// Create a new chat client
    pub fn new(user_id: String) -> Self {
        Self {
            user_id,
            current_room: None,
            rooms: Vec::new(),
            message_cache: HashMap::new(),
            server_sender: None,
        }
    }

    /// Send a text message
    pub async fn send_text(&self, room_id: String, text: String, reply_to: Option<String>) -> Result<(), ChatError> {
        if let Some(sender) = &self.server_sender {
            sender.send(ChatProtocolMessage::SendMessage {
                room_id,
                content: MessageContent::Text(text),
                reply_to,
            }).await.map_err(|_| ChatError::ConnectionClosed)?;
        }
        Ok(())
    }

    /// Set current room
    pub fn set_current_room(&mut self, room_id: Option<String>) {
        self.current_room = room_id;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_config_default() {
        let config = ChatServerConfig::default();
        assert!(config.enable_file_transfer);
    }
}
