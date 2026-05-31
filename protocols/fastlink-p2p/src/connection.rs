//! Connection Management
//!
//! FastLink P2P connection handling and management

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use thiserror::Error;
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::sync::{mpsc, RwLock};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{debug, info, warn, error};
use serde::{Deserialize, Serialize};
use bytes::{Bytes, BytesMut, Buf, BufMut};

use super::node::NodeId;

const MAX_MESSAGE_SIZE: usize = 10 * 1024 * 1024; // 10 MB
const MESSAGE_QUEUE_SIZE: usize = 1000;

/// Connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionState {
    Connecting,
    Connected,
    Disconnected,
    Error,
}

/// Protocol for a connection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionProtocol {
    Tcp,
    Udp,
    WebSocket,
}

/// Connection info
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub node_id: NodeId,
    pub local_addr: SocketAddr,
    pub remote_addr: SocketAddr,
    pub protocol: ConnectionProtocol,
    pub state: ConnectionState,
    pub created_at: u64,
    pub last_message_at: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub messages_sent: u64,
    pub messages_received: u64,
}

impl ConnectionInfo {
    fn new(node_id: NodeId, local_addr: SocketAddr, remote_addr: SocketAddr, protocol: ConnectionProtocol) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Self {
            node_id,
            local_addr,
            remote_addr,
            protocol,
            state: ConnectionState::Connecting,
            created_at: now,
            last_message_at: now,
            bytes_sent: 0,
            bytes_received: 0,
            messages_sent: 0,
            messages_received: 0,
        }
    }
}

/// Message frame for wire protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageFrame {
    pub magic: u32,
    pub version: u8,
    pub message_type: MessageType,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    Handshake = 0x01,
    HandshakeAck = 0x02,
    Data = 0x03,
    Ack = 0x04,
    Ping = 0x05,
    Pong = 0x06,
    Close = 0x07,
}

impl MessageFrame {
    const MAGIC: u32 = 0x464C5043; // "FLPC" - FastLink Protocol Connection
    const VERSION: u8 = 1;
    
    pub fn new(message_type: MessageType, payload: Vec<u8>) -> Self {
        Self {
            magic: Self::MAGIC,
            version: Self::VERSION,
            message_type,
            payload,
        }
    }
    
    pub fn encode(&self) -> Result<Vec<u8>, ConnectionError> {
        let mut buf = BytesMut::new();
        
        buf.put_u32(self.magic);
        buf.put_u8(self.version);
        buf.put_u8(self.message_type as u8);
        buf.put_u32(self.payload.len() as u32);
        buf.put_slice(&self.payload);
        
        Ok(buf.to_vec())
    }
    
    pub fn decode(data: &[u8]) -> Result<Self, ConnectionError> {
        if data.len() < 10 {
            return Err(ConnectionError::InvalidFrame);
        }
        
        let mut buf = Bytes::copy_from_slice(data);
        
        let magic = buf.get_u32();
        if magic != Self::MAGIC {
            return Err(ConnectionError::InvalidFrame);
        }
        
        let version = buf.get_u8();
        let message_type_byte = buf.get_u8();
        let message_type = match message_type_byte {
            0x01 => MessageType::Handshake,
            0x02 => MessageType::HandshakeAck,
            0x03 => MessageType::Data,
            0x04 => MessageType::Ack,
            0x05 => MessageType::Ping,
            0x06 => MessageType::Pong,
            0x07 => MessageType::Close,
            _ => return Err(ConnectionError::InvalidFrame),
        };
        
        let payload_len = buf.get_u32() as usize;
        if payload_len > MAX_MESSAGE_SIZE {
            return Err(ConnectionError::MessageTooLarge);
        }
        
        let mut payload = vec![0u8; payload_len];
        buf.copy_to_slice(&mut payload);
        
        Ok(Self {
            magic,
            version,
            message_type,
            payload,
        })
    }
}

/// P2P connection
pub struct P2PConnection {
    info: ConnectionInfo,
    send_tx: mpsc::Sender<Vec<u8>>,
    recv_rx: mpsc::Receiver<Vec<u8>>,
    close_tx: Option<mpsc::Sender<()>>,
}

impl P2PConnection {
    /// Create a new P2P connection from a TCP stream
    pub async fn from_tcp(
        mut stream: TcpStream,
        local_node_id: NodeId,
        remote_addr: SocketAddr,
        is_initiator: bool,
    ) -> Result<Self, ConnectionError> {
        let local_addr = stream.local_addr()?;
        
        let (read_half, write_half) = stream.into_split();
        
        let (send_tx, send_rx) = mpsc::channel(MESSAGE_QUEUE_SIZE);
        let (recv_tx, recv_rx) = mpsc::channel(MESSAGE_QUEUE_SIZE);
        let (close_tx, close_rx) = mpsc::channel(1);
        
        let info = ConnectionInfo::new(
            NodeId::new(),
            local_addr,
            remote_addr,
            ConnectionProtocol::Tcp,
        );
        
        let info_for_read = info.clone();
        let info_for_write = info.clone();
        
        tokio::spawn(async move {
            if let Err(e) = Self::read_loop(read_half, recv_tx, close_rx, info_for_read).await {
                error!("Read loop error: {}", e);
            }
        });
        
        tokio::spawn(async move {
            if let Err(e) = Self::write_loop(write_half, send_rx, info_for_write).await {
                error!("Write loop error: {}", e);
            }
        });
        
        Ok(Self {
            info,
            send_tx,
            recv_rx,
            close_tx: Some(close_tx),
        })
    }
    
    async fn read_loop(
        mut read_half: tokio::net::tcp::OwnedReadHalf,
        recv_tx: mpsc::Sender<Vec<u8>>,
        mut close_rx: mpsc::Receiver<()>,
        mut info: ConnectionInfo,
    ) -> Result<(), ConnectionError> {
        let mut buf = BytesMut::with_capacity(64 * 1024);
        
        loop {
            tokio::select! {
                _ = close_rx.recv() => {
                    info!("Read loop closed");
                    break Ok(());
                }
                
                result = read_half.read_buf(&mut buf) => {
                    match result {
                        Ok(0) => {
                            warn!("Connection closed by remote");
                            break Err(ConnectionError::ConnectionClosed);
                        }
                        Ok(n) => {
                            debug!("Read {} bytes", n);
                            info.bytes_received += n as u64;
                            
                            let mut should_close = false;
                            while buf.len() >= 10 {
                                let len_buf = &buf[8..10];
                                let payload_len = u32::from_be_bytes([0, 0, len_buf[0], len_buf[1]]) as usize;
                                
                                if buf.len() < 10 + payload_len {
                                    break;
                                }
                                
                                let frame_data = buf.split_to(10 + payload_len);
                                match MessageFrame::decode(&frame_data) {
                                    Ok(frame) => {
                                        if frame.message_type == MessageType::Data {
                                            if recv_tx.send(frame.payload).await.is_err() {
                                                should_close = true;
                                                break;
                                            }
                                            info.messages_received += 1;
                                        }
                                    }
                                    Err(e) => {
                                        warn!("Failed to decode frame: {}", e);
                                    }
                                }
                            }
                            
                            if should_close {
                                break Err(ConnectionError::ConnectionClosed);
                            }
                        }
                        Err(e) => {
                            error!("Read error: {}", e);
                            break Err(ConnectionError::Io(e));
                        }
                    }
                }
            }
        }
    }
    
    async fn write_loop(
        mut write_half: tokio::net::tcp::OwnedWriteHalf,
        mut send_rx: mpsc::Receiver<Vec<u8>>,
        mut info: ConnectionInfo,
    ) -> Result<(), ConnectionError> {
        while let Some(data) = send_rx.recv().await {
            let frame = MessageFrame::new(MessageType::Data, data);
            let encoded = frame.encode()?;
            
            write_half.write_all(&encoded).await?;
            info.bytes_sent += encoded.len() as u64;
            info.messages_sent += 1;
        }
        
        Ok(())
    }
    
    /// Get connection info
    pub fn info(&self) -> &ConnectionInfo {
        &self.info
    }
    
    /// Send a message through the connection
    pub async fn send(&self, data: Vec<u8>) -> Result<(), ConnectionError> {
        if data.len() > MAX_MESSAGE_SIZE {
            return Err(ConnectionError::MessageTooLarge);
        }
        
        self.send_tx.send(data).await
            .map_err(|_| ConnectionError::ConnectionClosed)
    }
    
    /// Receive a message from the connection
    pub async fn recv(&mut self) -> Option<Vec<u8>> {
        self.recv_rx.recv().await
    }
    
    /// Close the connection
    pub async fn close(&mut self) {
        if let Some(close_tx) = self.close_tx.take() {
            let _ = close_tx.send(()).await;
        }
        self.info.state = ConnectionState::Disconnected;
    }
}

#[derive(Debug, Error)]
pub enum ConnectionError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Connection closed")]
    ConnectionClosed,
    #[error("Maximum connections reached")]
    MaxConnectionsReached,
    #[error("Handshake failed: {0}")]
    HandshakeFailed(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Invalid frame")]
    InvalidFrame,
    #[error("Message too large")]
    MessageTooLarge,
}

/// Connection manager - manages all active P2P connections
pub struct ConnectionManager {
    connections: Arc<RwLock<HashMap<NodeId, P2PConnection>>>,
    max_connections: usize,
}

impl ConnectionManager {
    /// Create a new connection manager
    pub fn new(max_connections: usize, _message_queue_size: usize) -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            max_connections,
        }
    }
    
    /// Get the number of active connections
    pub async fn count(&self) -> usize {
        self.connections.read().await.len()
    }
    
    /// Check if we have a connection to a node
    pub async fn has_connection(&self, node_id: &NodeId) -> bool {
        self.connections.read().await.contains_key(node_id)
    }
    
    /// Send a message to a specific node
    pub async fn send_to(&self, node_id: &NodeId, data: Vec<u8>) -> Result<(), ConnectionError> {
        let connections = self.connections.read().await;
        if let Some(conn) = connections.get(node_id) {
            conn.send(data).await
        } else {
            Err(ConnectionError::ConnectionClosed)
        }
    }
    
    /// Broadcast a message to all connected nodes
    pub async fn broadcast(&self, data: Vec<u8>) -> Result<usize, ConnectionError> {
        let connections = self.connections.read().await;
        let mut success_count = 0;
        
        for conn in connections.values() {
            if conn.send(data.clone()).await.is_ok() {
                success_count += 1;
            }
        }
        
        Ok(success_count)
    }
    
    /// Add a new connection
    pub async fn add(
        &self,
        node_id: NodeId,
        conn: P2PConnection,
    ) -> Result<(), ConnectionError> {
        let mut guard = self.connections.write().await;
        
        if guard.len() >= self.max_connections {
            return Err(ConnectionError::MaxConnectionsReached);
        }
        
        guard.insert(node_id, conn);
        
        info!("Connected to node: {:?}", node_id);
        
        Ok(())
    }
    
    /// Remove a connection
    pub async fn remove(&self, node_id: &NodeId) {
        if let Some(mut conn) = self.connections.write().await.remove(node_id) {
            conn.close().await;
            info!("Disconnected from node: {:?}", node_id);
        }
    }
    
    /// Get all connection info
    pub async fn get_all_info(&self) -> Vec<ConnectionInfo> {
        self.connections
            .read()
            .await
            .values()
            .map(|c| c.info.clone())
            .collect()
    }
    
    /// Close all connections
    pub async fn close_all(&self) {
        let mut connections = self.connections.write().await;
        for (node_id, mut conn) in connections.drain() {
            conn.close().await;
            info!("Closed connection to node: {:?}", node_id);
        }
    }
}

/// P2P listener for incoming connections
pub struct P2PListener {
    local_addr: SocketAddr,
    tcp_listener: Option<TcpListener>,
    connection_manager: Arc<ConnectionManager>,
    local_node_id: NodeId,
}

impl P2PListener {
    /// Create a new listener
    pub fn new(
        local_addr: SocketAddr,
        connection_manager: Arc<ConnectionManager>,
        local_node_id: NodeId,
    ) -> Self {
        Self {
            local_addr,
            tcp_listener: None,
            connection_manager,
            local_node_id,
        }
    }
    
    /// Start listening for incoming connections
    pub async fn start(&mut self) -> Result<(), ConnectionError> {
        let tcp_listener = TcpListener::bind(self.local_addr).await?;
        self.local_addr = tcp_listener.local_addr()?;
        self.tcp_listener = Some(tcp_listener);
        
        info!("P2P listener started on {:?}", self.local_addr);
        
        Ok(())
    }
    
    /// Accept incoming connections in a loop
    pub async fn accept_loop(&mut self) {
        if let Some(ref tcp_listener) = self.tcp_listener {
            loop {
                match tcp_listener.accept().await {
                    Ok((stream, addr)) => {
                        debug!("Incoming connection from {:?}", addr);
                        
                        let conn_manager = self.connection_manager.clone();
                        let local_node_id = self.local_node_id;
                        
                        tokio::spawn(async move {
                            match P2PConnection::from_tcp(stream, local_node_id, addr, false).await {
                                Ok(conn) => {
                                    let node_id = conn.info().node_id;
                                    if let Err(e) = conn_manager.add(node_id, conn).await {
                                        warn!("Failed to add connection: {}", e);
                                    }
                                }
                                Err(e) => {
                                    warn!("Failed to establish connection from {}: {}", addr, e);
                                }
                            }
                        });
                    }
                    Err(e) => {
                        warn!("Error accepting connection: {:?}", e);
                    }
                }
            }
        }
    }
    
    /// Get the local address
    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }
    
    /// Stop the listener
    pub async fn stop(&mut self) {
        self.tcp_listener = None;
        info!("P2P listener stopped");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, SocketAddrV4};
    
    #[test]
    fn test_connection_manager_creation() {
        let manager = ConnectionManager::new(100, 1000);
        assert_eq!(tokio::runtime::Runtime::new().unwrap().block_on(manager.count()), 0);
    }
    
    #[test]
    fn test_message_frame_encoding() {
        let frame = MessageFrame::new(MessageType::Data, vec![1, 2, 3, 4]);
        let encoded = frame.encode().unwrap();
        assert!(!encoded.is_empty());
        
        let decoded = MessageFrame::decode(&encoded).unwrap();
        assert_eq!(decoded.message_type, MessageType::Data);
        assert_eq!(decoded.payload, vec![1, 2, 3, 4]);
    }
    
    #[test]
    fn test_connection_info_creation() {
        let node_id = NodeId::new();
        let local_addr: SocketAddr = ([127, 0, 0, 1], 8080).into();
        let remote_addr: SocketAddr = ([127, 0, 0, 1], 9090).into();
        
        let info = ConnectionInfo::new(node_id, local_addr, remote_addr, ConnectionProtocol::Tcp);
        assert_eq!(info.protocol, ConnectionProtocol::Tcp);
        assert_eq!(info.state, ConnectionState::Connecting);
    }
}
