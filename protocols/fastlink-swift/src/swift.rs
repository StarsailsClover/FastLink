//! FastLink-Swift Protocol
//!
//! A high-performance, QUIC-based transport protocol for low-latency data transfer.
//! Provides reliable and unreliable transport modes with congestion control.

use std::collections::{VecDeque, HashMap};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::{mpsc, RwLock};
use tokio::net::UdpSocket;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::{timeout, interval};
use tracing::{debug, info, warn, error};
use serde::{Deserialize, Serialize};
use bytes::{Bytes, BytesMut, Buf, BufMut};
use rand::Rng;

const MAX_FRAME_SIZE: usize = 1350; // MTU friendly
const INITIAL_WINDOW: u64 = 65535;
const MAX_STREAMS: usize = 100;
const KEEPALIVE_INTERVAL_MS: u64 = 5000;
const IDLE_TIMEOUT_MS: u64 = 30000;

/// Swift protocol error type
#[derive(Debug, Error)]
pub enum SwiftError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Connection closed")]
    ConnectionClosed,
    #[error("Timeout")]
    Timeout,
    #[error("Invalid frame")]
    InvalidFrame,
    #[error("Handshake failed: {0}")]
    HandshakeFailed(String),
    #[error("Stream not found")]
    StreamNotFound,
    #[error("Max streams reached")]
    MaxStreamsReached,
}

/// Swift connection mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransportMode {
    /// Reliable ordered delivery
    Reliable,
    /// Unreliable unordered delivery
    Unreliable,
    /// Partially reliable
    PartiallyReliable,
}

/// Swift frame type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SwiftFrame {
    Handshake {
        version: u32,
        connection_id: u64,
        supported_modes: Vec<TransportMode>,
    },
    HandshakeAck {
        version: u32,
        connection_id: u64,
        selected_mode: TransportMode,
    },
    Data {
        stream_id: u32,
        sequence: u64,
        offset: u64,
        fin: bool,
        data: Vec<u8>,
    },
    Ack {
        ack_ranges: Vec<AckRange>,
    },
    StreamReset {
        stream_id: u32,
        error_code: u32,
    },
    Ping {
        timestamp: u64,
    },
    Pong {
        timestamp: u64,
    },
    Close {
        error_code: u32,
        reason: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AckRange {
    pub first: u64,
    pub last: u64,
}

impl SwiftFrame {
    pub fn encode(&self) -> Result<Vec<u8>, SwiftError> {
        let serialized = bincode::serialize(self)
            .map_err(|e| SwiftError::InvalidFrame)?;
        Ok(serialized)
    }
    
    pub fn decode(data: &[u8]) -> Result<Self, SwiftError> {
        bincode::deserialize(data)
            .map_err(|e| SwiftError::InvalidFrame)
    }
}

/// Swift connection configuration
#[derive(Debug, Clone)]
pub struct SwiftConfig {
    pub local_addr: Option<SocketAddr>,
    pub transport_mode: TransportMode,
    pub max_concurrent_streams: usize,
    pub initial_window_size: u64,
    pub idle_timeout_ms: u64,
    pub keep_alive_interval_ms: u64,
}

impl Default for SwiftConfig {
    fn default() -> Self {
        Self {
            local_addr: None,
            transport_mode: TransportMode::Reliable,
            max_concurrent_streams: MAX_STREAMS,
            initial_window_size: INITIAL_WINDOW,
            idle_timeout_ms: IDLE_TIMEOUT_MS,
            keep_alive_interval_ms: KEEPALIVE_INTERVAL_MS,
        }
    }
}

/// Stream state
#[derive(Debug)]
struct StreamState {
    stream_id: u32,
    next_send_seq: u64,
    next_recv_seq: u64,
    send_offset: u64,
    recv_offset: u64,
    send_window: u64,
    recv_window: u64,
    send_buffer: VecDeque<(u64, Vec<u8>)>,
    recv_buffer: HashMap<u64, Vec<u8>>,
    is_closed: bool,
}

impl StreamState {
    fn new(stream_id: u32) -> Self {
        Self {
            stream_id,
            next_send_seq: 0,
            next_recv_seq: 0,
            send_offset: 0,
            recv_offset: 0,
            send_window: INITIAL_WINDOW,
            recv_window: INITIAL_WINDOW,
            send_buffer: VecDeque::new(),
            recv_buffer: HashMap::new(),
            is_closed: false,
        }
    }
}

/// Congestion controller
#[derive(Debug)]
struct CongestionController {
    cwnd: u64,
    ssthresh: u64,
    bytes_in_flight: u64,
    rtt: Duration,
    rtt_var: Duration,
    min_rtt: Duration,
    last_update: Instant,
}

impl CongestionController {
    fn new() -> Self {
        Self {
            cwnd: INITIAL_WINDOW,
            ssthresh: u64::MAX,
            bytes_in_flight: 0,
            rtt: Duration::from_millis(100),
            rtt_var: Duration::from_millis(50),
            min_rtt: Duration::from_secs(1),
            last_update: Instant::now(),
        }
    }
    
    fn on_ack(&mut self, _bytes: u64) {
        // Simple slow start / congestion avoidance
        if self.cwnd < self.ssthresh {
            self.cwnd += 1460; // Slow start
        } else {
            self.cwnd += 1460 / self.cwnd; // Congestion avoidance
        }
    }
    
    fn on_loss(&mut self) {
        self.ssthresh = self.cwnd / 2;
        self.cwnd = INITIAL_WINDOW;
    }
    
    fn can_send(&self) -> bool {
        self.bytes_in_flight < self.cwnd
    }
}

/// Swift connection
pub struct SwiftConnection {
    config: SwiftConfig,
    local_addr: SocketAddr,
    remote_addr: SocketAddr,
    connection_id: u64,
    streams: Arc<RwLock<HashMap<u32, StreamState>>>,
    congestion: Arc<RwLock<CongestionController>>,
    send_queue: mpsc::Sender<(SocketAddr, Vec<u8>)>,
    state: Arc<RwLock<ConnectionState>>,
    last_activity: Arc<RwLock<Instant>>,
}

/// Connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Idle,
    Connecting,
    Connected,
    Closing,
    Closed,
}

impl SwiftConnection {
    /// Create a new Swift connection
    pub async fn new(
        config: SwiftConfig,
        socket: Arc<UdpSocket>,
        remote_addr: SocketAddr,
        is_initiator: bool,
    ) -> Result<Self, SwiftError> {
        let local_addr = socket.local_addr()?;
        let connection_id = rand::thread_rng().gen();
        
        let (send_tx, mut send_rx) = mpsc::channel::<(SocketAddr, Vec<u8>)>(1000);
        let streams = Arc::new(RwLock::new(HashMap::new()));
        let congestion = Arc::new(RwLock::new(CongestionController::new()));
        let state = Arc::new(RwLock::new(ConnectionState::Idle));
        let last_activity = Arc::new(RwLock::new(Instant::now()));
        
        // Spawn send task
        let socket_clone = socket.clone();
        tokio::spawn(async move {
            while let Some((addr, data)) = send_rx.recv().await {
                if let Err(e) = socket_clone.send_to(&data, addr).await {
                    error!("Failed to send to {}: {}", addr, e);
                }
            }
        });
        
        let conn = Self {
            config,
            local_addr,
            remote_addr,
            connection_id,
            streams,
            congestion,
            send_queue: send_tx,
            state,
            last_activity,
        };
        
        // Perform handshake
        if is_initiator {
            conn.perform_handshake().await?;
        }
        
        Ok(conn)
    }
    
    async fn perform_handshake(&self) -> Result<(), SwiftError> {
        *self.state.write().await = ConnectionState::Connecting;
        
        let handshake = SwiftFrame::Handshake {
            version: 1,
            connection_id: self.connection_id,
            supported_modes: vec![self.config.transport_mode],
        };
        
        let encoded = handshake.encode()?;
        self.send_queue.send((self.remote_addr, encoded)).await
            .map_err(|_| SwiftError::ConnectionClosed)?;
        
        // Wait for handshake ack with timeout
        // TODO: Implement actual handshake response handling
        
        *self.state.write().await = ConnectionState::Connected;
        Ok(())
    }
    
    /// Get connection state
    pub async fn state(&self) -> ConnectionState {
        *self.state.read().await
    }
    
    /// Get local address
    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }
    
    /// Get remote address
    pub fn remote_addr(&self) -> SocketAddr {
        self.remote_addr
    }
    
    /// Open a new stream
    pub async fn open_stream(&self) -> Result<u32, SwiftError> {
        let mut streams = self.streams.write().await;
        
        if streams.len() >= self.config.max_concurrent_streams {
            return Err(SwiftError::MaxStreamsReached);
        }
        
        let stream_id = streams.len() as u32;
        streams.insert(stream_id, StreamState::new(stream_id));
        
        Ok(stream_id)
    }
    
    /// Send data through a stream
    pub async fn send(&self, stream_id: u32, data: Vec<u8>) -> Result<(), SwiftError> {
        let mut streams = self.streams.write().await;
        let stream = streams.get_mut(&stream_id)
            .ok_or(SwiftError::StreamNotFound)?;
        
        if stream.is_closed {
            return Err(SwiftError::ConnectionClosed);
        }
        
        let seq = stream.next_send_seq;
        stream.next_send_seq += 1;
        
        let frame = SwiftFrame::Data {
            stream_id,
            sequence: seq,
            offset: stream.send_offset,
            fin: false,
            data: data.clone(),
        };
        
        stream.send_offset += data.len() as u64;
        stream.send_buffer.push_back((seq, data));
        
        let encoded = frame.encode()?;
        self.send_queue.send((self.remote_addr, encoded)).await
            .map_err(|_| SwiftError::ConnectionClosed)?;
        
        *self.last_activity.write().await = Instant::now();
        
        Ok(())
    }
    
    /// Receive data from any stream
    pub async fn recv(&self) -> Result<(u32, Vec<u8>), SwiftError> {
        // Check for available data in streams
        let streams = self.streams.read().await;
        let mut found_data: Option<(u32, u64, Vec<u8>)> = None;
        
        for (stream_id, state) in streams.iter() {
            if let Some(data) = state.recv_buffer.get(&state.next_recv_seq) {
                found_data = Some((*stream_id, state.next_recv_seq, data.clone()));
                break;
            }
        }
        
        drop(streams);
        
        if let Some((stream_id, seq, data)) = found_data {
            let mut streams = self.streams.write().await;
            if let Some(stream) = streams.get_mut(&stream_id) {
                stream.recv_buffer.remove(&seq);
                stream.next_recv_seq += 1;
            }
            
            return Ok((stream_id, data));
        }
        
        Err(SwiftError::ConnectionClosed)
    }
    
    /// Handle incoming frame
    pub async fn handle_frame(&self, frame: SwiftFrame) -> Result<(), SwiftError> {
        *self.last_activity.write().await = Instant::now();
        
        match frame {
            SwiftFrame::Data { stream_id, sequence, offset: _, fin, data } => {
                let mut streams = self.streams.write().await;
                let stream = streams.entry(stream_id).or_insert_with(|| StreamState::new(stream_id));
                stream.recv_buffer.insert(sequence, data);
                
                // Send ACK
                let ack = SwiftFrame::Ack {
                    ack_ranges: vec![AckRange {
                        first: sequence,
                        last: sequence,
                    }],
                };
                let encoded = ack.encode()?;
                self.send_queue.send((self.remote_addr, encoded)).await
                    .map_err(|_| SwiftError::ConnectionClosed)?;
                
                if fin {
                    stream.is_closed = true;
                }
            }
            SwiftFrame::Ack { ack_ranges } => {
                let mut congestion = self.congestion.write().await;
                for range in ack_ranges {
                    congestion.on_ack(range.last - range.first + 1);
                }
            }
            SwiftFrame::Ping { timestamp } => {
                let pong = SwiftFrame::Pong { timestamp };
                let encoded = pong.encode()?;
                self.send_queue.send((self.remote_addr, encoded)).await
                    .map_err(|_| SwiftError::ConnectionClosed)?;
            }
            SwiftFrame::Close { error_code, reason } => {
                info!("Connection closed by remote: {} - {}", error_code, reason);
                *self.state.write().await = ConnectionState::Closed;
            }
            _ => {}
        }
        
        Ok(())
    }
    
    /// Close the connection
    pub async fn close(&self, reason: String) -> Result<(), SwiftError> {
        *self.state.write().await = ConnectionState::Closing;
        
        let frame = SwiftFrame::Close {
            error_code: 0,
            reason,
        };
        
        let encoded = frame.encode()?;
        let _ = self.send_queue.send((self.remote_addr, encoded)).await;
        
        *self.state.write().await = ConnectionState::Closed;
        Ok(())
    }
}

/// Swift server - accepts incoming Swift connections
pub struct SwiftServer {
    config: SwiftConfig,
    socket: Option<Arc<UdpSocket>>,
    connections: Arc<RwLock<HashMap<SocketAddr, SwiftConnection>>>,
}

impl SwiftServer {
    /// Create a new Swift server
    pub fn new(config: SwiftConfig) -> Self {
        Self {
            config,
            socket: None,
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Start the server and listen for connections
    pub async fn start(&mut self) -> Result<(), SwiftError> {
        let addr = self.config.local_addr.unwrap_or(([0, 0, 0, 0], 0).into());
        let socket = UdpSocket::bind(addr).await?;
        let local_addr = socket.local_addr()?;
        
        info!("Swift server listening on {}", local_addr);
        
        let socket = Arc::new(socket);
        self.socket = Some(socket.clone());
        
        // Start receive loop
        let connections = self.connections.clone();
        let config = self.config.clone();
        
        tokio::spawn(async move {
            let mut buf = vec![0u8; 65535];
            
            loop {
                match socket.recv_from(&mut buf).await {
                    Ok((len, addr)) => {
                        if let Ok(frame) = SwiftFrame::decode(&buf[..len]) {
                            if let SwiftFrame::Handshake { version, connection_id, supported_modes } = frame {
                                // Create new connection
                                let conn_result = SwiftConnection::new(
                                    config.clone(),
                                    socket.clone(),
                                    addr,
                                    false,
                                ).await;
                                
                                if let Ok(conn) = conn_result {
                                    connections.write().await.insert(addr, conn);
                                    info!("New Swift connection from {}", addr);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Receive error: {}", e);
                    }
                }
            }
        });
        
        Ok(())
    }
    
    /// Stop the server
    pub async fn stop(&self) {
        info!("Stopping Swift server");
        let mut connections = self.connections.write().await;
        for (_, conn) in connections.drain() {
            let _ = conn.close("Server shutdown".to_string()).await;
        }
    }
}

/// Swift client - initiates Swift connections
pub struct SwiftClient {
    config: SwiftConfig,
    socket: Option<Arc<UdpSocket>>,
}

impl SwiftClient {
    /// Create a new Swift client
    pub fn new(config: SwiftConfig) -> Self {
        Self {
            config,
            socket: None,
        }
    }
    
    /// Connect to a Swift server
    pub async fn connect(&mut self, server_addr: SocketAddr) -> Result<SwiftConnection, SwiftError> {
        let local_addr: SocketAddr = ([0, 0, 0, 0], 0).into();
        let socket = UdpSocket::bind(local_addr).await?;
        let socket = Arc::new(socket);
        self.socket = Some(socket.clone());
        
        debug!("Connecting to Swift server at {}", server_addr);
        
        SwiftConnection::new(self.config.clone(), socket, server_addr, true).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_swift_config_default() {
        let config = SwiftConfig::default();
        assert_eq!(config.transport_mode, TransportMode::Reliable);
        assert_eq!(config.max_concurrent_streams, MAX_STREAMS);
    }
    
    #[test]
    fn test_frame_encoding() {
        let frame = SwiftFrame::Ping { timestamp: 12345 };
        let encoded = frame.encode().unwrap();
        let decoded = SwiftFrame::decode(&encoded).unwrap();
        
        match decoded {
            SwiftFrame::Ping { timestamp } => assert_eq!(timestamp, 12345),
            _ => panic!("Wrong frame type"),
        }
    }
    
    #[test]
    fn test_congestion_controller() {
        let mut cc = CongestionController::new();
        assert!(cc.can_send());
        
        cc.on_ack(1460);
        assert!(cc.cwnd > INITIAL_WINDOW);
        
        cc.on_loss();
        assert!(cc.cwnd < INITIAL_WINDOW * 2);
    }
}
