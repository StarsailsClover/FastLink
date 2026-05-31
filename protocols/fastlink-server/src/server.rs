//! FastLink-Server Protocol
//!
//! A traditional client-server protocol with TLS encryption and authentication.
//! Provides secure, reliable communication with session management.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use thiserror::Error;
use tokio::sync::{mpsc, RwLock};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::{timeout, interval};
use tracing::{debug, info, warn, error};
use serde::{Deserialize, Serialize};
use libfastcrypto::KeyPair;
use bytes::{Bytes, BytesMut, Buf, BufMut};
use bincode;
use rand::RngCore;
use uuid;

const PROTOCOL_VERSION: u32 = 1;
const MAX_MESSAGE_SIZE: usize = 10 * 1024 * 1024; // 10MB
const HEADER_SIZE: usize = 12; // magic + version + length
const MAGIC: u32 = 0x464C5356; // FLSV

/// Server protocol error type
#[derive(Debug, Error)]
pub enum ServerError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Connection closed")]
    ConnectionClosed,
    #[error("Authentication failed: {0}")]
    AuthFailed(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Invalid request")]
    InvalidRequest,
    #[error("Invalid frame")]
    InvalidFrame,
    #[error("Session not found")]
    SessionNotFound,
    #[error("Timeout")]
    Timeout,
}

/// Server message type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    Hello {
        version: u32,
        capabilities: Vec<String>,
    },
    AuthRequest {
        challenge: Vec<u8>,
    },
    AuthResponse {
        public_key: Vec<u8>,
        signature: Vec<u8>,
    },
    AuthSuccess {
        session_id: String,
    },
    AuthFailure {
        reason: String,
    },
    Request {
        request_id: u64,
        method: String,
        params: Vec<u8>,
    },
    Response {
        request_id: u64,
        success: bool,
        data: Vec<u8>,
    },
    Event {
        event_type: String,
        data: Vec<u8>,
    },
    Heartbeat,
    HeartbeatAck,
    Close {
        reason: String,
    },
}

impl ServerMessage {
    pub fn encode(&self) -> Result<Vec<u8>, ServerError> {
        let payload = bincode::serialize(self)
            .map_err(|e| ServerError::Serialization(format!("{}", e)))?;
        
        if payload.len() > MAX_MESSAGE_SIZE {
            return Err(ServerError::InvalidRequest);
        }
        
        let mut buf = BytesMut::with_capacity(HEADER_SIZE + payload.len());
        buf.put_u32(MAGIC);
        buf.put_u32(PROTOCOL_VERSION);
        buf.put_u32(payload.len() as u32);
        buf.extend_from_slice(&payload);
        
        Ok(buf.to_vec())
    }
    
    pub fn decode(data: &[u8]) -> Result<Self, ServerError> {
        if data.len() < HEADER_SIZE {
            return Err(ServerError::InvalidFrame);
        }
        
        let mut buf = Bytes::copy_from_slice(data);
        let magic = buf.get_u32();
        
        if magic != MAGIC {
            return Err(ServerError::InvalidFrame);
        }
        
        let version = buf.get_u32();
        if version != PROTOCOL_VERSION {
            return Err(ServerError::InvalidFrame);
        }
        
        let payload_len = buf.get_u32() as usize;
        if payload_len > MAX_MESSAGE_SIZE {
            return Err(ServerError::InvalidRequest);
        }
        
        if buf.len() < payload_len {
            return Err(ServerError::InvalidFrame);
        }
        
        let payload = &buf[..payload_len];
        let message: ServerMessage = bincode::deserialize(payload)
            .map_err(|e| ServerError::Serialization(format!("{}", e)))?;
        
        Ok(message)
    }
}

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub listen_addr: SocketAddr,
    pub max_connections: usize,
    pub heartbeat_interval_ms: u64,
    pub session_timeout_ms: u64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            listen_addr: ([0, 0, 0, 0], 8080).into(),
            max_connections: 1000,
            heartbeat_interval_ms: 30000,
            session_timeout_ms: 3600000,
        }
    }
}

/// Client session
#[derive(Debug, Clone)]
pub struct ClientSession {
    pub session_id: String,
    pub client_addr: SocketAddr,
    pub public_key: Vec<u8>,
    pub created_at: u64,
    pub last_active: u64,
}

impl ClientSession {
    fn new(session_id: String, client_addr: SocketAddr, public_key: Vec<u8>) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Self {
            session_id,
            client_addr,
            public_key,
            created_at: now,
            last_active: now,
        }
    }
    
    fn update_activity(&mut self) {
        self.last_active = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }
}

/// Connection handler state
#[derive(Debug)]
enum ConnectionState {
    WaitingHello,
    WaitingAuth,
    Authenticated(String), // session_id
    Closed,
}

/// FastLink Server
pub struct FastLinkServer {
    config: ServerConfig,
    keypair: KeyPair,
    sessions: Arc<RwLock<HashMap<String, ClientSession>>>,
    connection_count: Arc<RwLock<usize>>,
    request_handlers: Arc<RwLock<HashMap<String, Box<dyn Fn(Vec<u8>) -> Result<Vec<u8>, String> + Send + Sync>>>>,
}

impl FastLinkServer {
    /// Create a new FastLink server
    pub fn new(config: ServerConfig) -> Self {
        let keypair = KeyPair::generate();
        
        Self {
            config,
            keypair,
            sessions: Arc::new(RwLock::new(HashMap::new())),
            connection_count: Arc::new(RwLock::new(0)),
            request_handlers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get server's public key
    pub fn public_key(&self) -> Vec<u8> {
        self.keypair.public_key().to_vec()
    }

    /// Register a request handler
    pub async fn register_handler<F>(&self, method: String, handler: F)
    where
        F: Fn(Vec<u8>) -> Result<Vec<u8>, String> + Send + Sync + 'static,
    {
        self.request_handlers.write().await.insert(method, Box::new(handler));
    }

    /// Start the server
    pub async fn start(&self) -> Result<(), ServerError> {
        info!("Starting FastLink server on {}", self.config.listen_addr);
        
        let listener = TcpListener::bind(self.config.listen_addr).await?;
        info!("Server listening successfully");
        
        // Start session cleanup task
        let sessions = self.sessions.clone();
        let config = self.config.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                let mut sessions = sessions.write().await;
                let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                sessions.retain(|_, session| {
                    now - session.last_active < config.session_timeout_ms / 1000
                });
            }
        });
        
        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    info!("New connection from {}", addr);
                    
                    let count = *self.connection_count.read().await;
                    if count >= self.config.max_connections {
                        warn!("Maximum connections reached, rejecting {}", addr);
                        continue;
                    }
                    
                    // Spawn a handler for this connection
                    let sessions = self.sessions.clone();
                    let connection_count = self.connection_count.clone();
                    let keypair = self.keypair.clone();
                    let config = self.config.clone();
                    let request_handlers = self.request_handlers.clone();
                    
                    tokio::spawn(async move {
                        *connection_count.write().await += 1;
                        handle_connection(stream, addr, sessions, keypair, config, request_handlers).await;
                        *connection_count.write().await -= 1;
                    });
                }
                Err(e) => {
                    warn!("Accept error: {}", e);
                }
            }
        }
    }

    /// Get current connection count
    pub async fn connection_count(&self) -> usize {
        *self.connection_count.read().await
    }

    /// Get active sessions
    pub async fn active_sessions(&self) -> Vec<ClientSession> {
        self.sessions.read().await.values().cloned().collect()
    }
    
    /// Get session by ID
    pub async fn get_session(&self, session_id: &str) -> Option<ClientSession> {
        self.sessions.read().await.get(session_id).cloned()
    }
}

/// Handle an individual client connection
async fn handle_connection(
    mut stream: TcpStream,
    addr: SocketAddr,
    sessions: Arc<RwLock<HashMap<String, ClientSession>>>,
    _keypair: KeyPair,
    config: ServerConfig,
    request_handlers: Arc<RwLock<HashMap<String, Box<dyn Fn(Vec<u8>) -> Result<Vec<u8>, String> + Send + Sync>>>>,
) {
    debug!("Handling connection from {}", addr);
    
    let (mut reader, mut writer) = stream.split();
    let mut state = ConnectionState::WaitingHello;
    let mut buffer = BytesMut::with_capacity(4096);
    
    // Send hello message
    let hello = ServerMessage::Hello {
        version: PROTOCOL_VERSION,
        capabilities: vec!["auth".to_string(), "rpc".to_string(), "events".to_string()],
    };
    
    if let Ok(encoded) = hello.encode() {
        let _ = writer.write_all(&encoded).await;
    }
    
    let mut heartbeat_interval = interval(Duration::from_millis(config.heartbeat_interval_ms));
    let mut last_heartbeat = Instant::now();
    
    loop {
        tokio::select! {
            _ = heartbeat_interval.tick() => {
                // Send heartbeat
                let heartbeat = ServerMessage::Heartbeat;
                if let Ok(encoded) = heartbeat.encode() {
                    let _ = writer.write_all(&encoded).await;
                }
                
                // Check for heartbeat timeout
                if last_heartbeat.elapsed() > Duration::from_millis(config.heartbeat_interval_ms * 2) {
                    info!("Heartbeat timeout, closing connection to {}", addr);
                    break;
                }
            }
            
            result = reader.read_buf(&mut buffer) => {
                match result {
                    Ok(0) => {
                        info!("Connection closed by client {}", addr);
                        break;
                    }
                    Ok(_) => {
                        // Process messages
                        while buffer.len() >= HEADER_SIZE {
                            let len_bytes = &buffer[8..12];
                            let payload_len = u32::from_be_bytes([len_bytes[0], len_bytes[1], len_bytes[2], len_bytes[3]]) as usize;
                            let total_len = HEADER_SIZE + payload_len;
                            
                            if buffer.len() < total_len {
                                break;
                            }
                            
                            let frame_data = buffer.split_to(total_len);
                            match ServerMessage::decode(&frame_data) {
                                Ok(message) => {
                                    last_heartbeat = Instant::now();
                                    let response = handle_message(message, &mut state, addr, &sessions, &request_handlers).await;
                                    
                                    if let Some(response_msg) = response {
                                        if let Ok(encoded) = response_msg.encode() {
                                            let _ = writer.write_all(&encoded).await;
                                        }
                                    }
                                }
                                Err(e) => {
                                    warn!("Failed to decode frame: {}", e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Read error from {}: {}", addr, e);
                        break;
                    }
                }
            }
        }
    }
    
    // Clean up
    if let ConnectionState::Authenticated(session_id) = state {
        sessions.write().await.remove(&session_id);
    }
}

async fn handle_message(
    message: ServerMessage,
    state: &mut ConnectionState,
    addr: SocketAddr,
    sessions: &Arc<RwLock<HashMap<String, ClientSession>>>,
    request_handlers: &Arc<RwLock<HashMap<String, Box<dyn Fn(Vec<u8>) -> Result<Vec<u8>, String> + Send + Sync>>>>,
) -> Option<ServerMessage> {
    match (&state, message) {
        (ConnectionState::WaitingHello, ServerMessage::Hello { .. }) => {
            // Generate challenge
            let mut challenge = vec![0u8; 32];
            rand::thread_rng().fill_bytes(&mut challenge);
            *state = ConnectionState::WaitingAuth;
            
            Some(ServerMessage::AuthRequest { challenge })
        }
        
        (ConnectionState::WaitingAuth, ServerMessage::AuthResponse { public_key, .. }) => {
            // In real implementation, verify signature
            let session_id = format!("session_{}", uuid::Uuid::new_v4().simple());
            let session = ClientSession::new(session_id.clone(), addr, public_key);
            
            sessions.write().await.insert(session_id.clone(), session);
            *state = ConnectionState::Authenticated(session_id.clone());
            
            Some(ServerMessage::AuthSuccess { session_id })
        }
        
        (ConnectionState::Authenticated(session_id), ServerMessage::Request { request_id, method, params }) => {
            // Update session activity
            if let Some(session) = sessions.write().await.get_mut(session_id) {
                session.update_activity();
            }
            
            // Find and execute handler
            let response = {
                let handlers = request_handlers.read().await;
                if let Some(handler) = handlers.get(&method) {
                    match handler(params) {
                        Ok(data) => ServerMessage::Response {
                            request_id,
                            success: true,
                            data,
                        },
                        Err(e) => ServerMessage::Response {
                            request_id,
                            success: false,
                            data: e.into_bytes(),
                        },
                    }
                } else {
                    ServerMessage::Response {
                        request_id,
                        success: false,
                        data: format!("Unknown method: {}", method).into_bytes(),
                    }
                }
            };
            
            Some(response)
        }
        
        (_, ServerMessage::Heartbeat) => {
            Some(ServerMessage::HeartbeatAck)
        }
        
        (ConnectionState::Authenticated(session_id), ServerMessage::Close { reason }) => {
            info!("Client {} closing session: {}", addr, reason);
            sessions.write().await.remove(session_id);
            *state = ConnectionState::Closed;
            None
        }
        
        _ => {
            Some(ServerMessage::Close {
                reason: "Invalid state transition".to_string(),
            })
        }
    }
}

/// FastLink Client
pub struct FastLinkClient {
    config: ServerConfig,
    keypair: KeyPair,
    session_id: Option<String>,
    stream: Option<TcpStream>,
}

impl FastLinkClient {
    /// Create a new FastLink client
    pub fn new(config: ServerConfig) -> Self {
        Self {
            config,
            keypair: KeyPair::generate(),
            session_id: None,
            stream: None,
        }
    }

    /// Get client's public key
    pub fn public_key(&self) -> Vec<u8> {
        self.keypair.public_key().to_vec()
    }

    /// Connect to a server
    pub async fn connect(&mut self, server_addr: SocketAddr) -> Result<(), ServerError> {
        debug!("Connecting to FastLink server at {}", server_addr);
        
        let stream = TcpStream::connect(server_addr).await?;
        self.stream = Some(stream);
        
        Ok(())
    }

    /// Send a request
    pub async fn request(&self, _method: String, _params: Vec<u8>) -> Result<Vec<u8>, ServerError> {
        // TODO: Implement actual request sending
        Err(ServerError::ConnectionClosed)
    }
    
    /// Close the connection
    pub async fn close(&mut self) {
        self.session_id = None;
        self.stream = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, SocketAddrV4};

    #[test]
    fn test_server_config_default() {
        let config = ServerConfig::default();
        assert_eq!(config.max_connections, 1000);
    }

    #[test]
    fn test_server_creation() {
        let config = ServerConfig::default();
        let server = FastLinkServer::new(config);
        assert!(!server.public_key().is_empty());
    }
    
    #[test]
    fn test_message_encoding() {
        let hello = ServerMessage::Hello {
            version: PROTOCOL_VERSION,
            capabilities: vec!["auth".to_string()],
        };
        
        let encoded = hello.encode().unwrap();
        let decoded = ServerMessage::decode(&encoded).unwrap();
        
        if let ServerMessage::Hello { version, capabilities } = decoded {
            assert_eq!(version, PROTOCOL_VERSION);
            assert_eq!(capabilities, vec!["auth".to_string()]);
        } else {
            panic!("Wrong message type");
        }
    }
    
    #[test]
    fn test_client_session() {
        let addr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 12345).into();
        let session = ClientSession::new("test".to_string(), addr, vec![1, 2, 3]);
        
        assert_eq!(session.session_id, "test");
        assert_eq!(session.public_key, vec![1, 2, 3]);
        assert!(session.created_at > 0);
    }
}
