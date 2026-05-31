//! FastLink transport module
//!
//! UDP-based transport with NAT traversal support

use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, Instant};
use std::collections::VecDeque;
use thiserror::Error;
use bytes::{Bytes, BytesMut};
use tracing::{debug, warn};

/// UDP-based connection abstraction
pub struct UdpTransport {
    socket: UdpSocket,
    send_buffer: VecDeque<(SocketAddr, Bytes)>,
    recv_buffer: VecDeque<(SocketAddr, Bytes)>,
    max_packet_size: usize,
}

impl UdpTransport {
    /// Create a new UDP transport
    pub fn bind(local_addr: SocketAddr) -> Result<Self, TransportError> {
        let socket = UdpSocket::bind(local_addr)?;
        socket.set_nonblocking(true)?;
        
        Ok(Self {
            socket,
            send_buffer: VecDeque::new(),
            recv_buffer: VecDeque::new(),
            max_packet_size: 65536,
        })
    }
    
    /// Get the local address
    pub fn local_addr(&self) -> Result<SocketAddr, TransportError> {
        Ok(self.socket.local_addr()?)
    }
    
    /// Send a packet to a destination
    pub fn send_to(&mut self, data: &[u8], dest: SocketAddr) -> Result<(), TransportError> {
        if data.len() > self.max_packet_size {
            return Err(TransportError::PacketTooLarge);
        }
        
        self.send_buffer.push_back((dest, Bytes::copy_from_slice(data)));
        self.flush()?;
        
        Ok(())
    }
    
    /// Flush the send buffer
    fn flush(&mut self) -> Result<(), TransportError> {
        while let Some((dest, data)) = self.send_buffer.pop_front() {
            self.socket.send_to(&data, dest)?;
        }
        Ok(())
    }
    
    /// Receive a packet
    pub fn recv_from(&mut self) -> Result<Option<(SocketAddr, Bytes)>, TransportError> {
        let mut buf = BytesMut::with_capacity(self.max_packet_size);
        buf.resize(self.max_packet_size, 0);
        
        match self.socket.recv_from(&mut buf) {
            Ok((size, addr)) => {
                buf.truncate(size);
                Ok(Some((addr, buf.freeze())))
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::WouldBlock {
                    Ok(None)
                } else {
                    Err(e.into())
                }
            }
        }
    }
    
    /// Process incoming packets and fill recv buffer
    pub fn poll(&mut self) -> Result<(), TransportError> {
        loop {
            match self.recv_from()? {
                Some(packet) => self.recv_buffer.push_back(packet),
                None => break,
            }
        }
        
        Ok(())
    }
    
    /// Get the next received packet
    pub fn next_packet(&mut self) -> Option<(SocketAddr, Bytes)> {
        self.recv_buffer.pop_front()
    }
    
    /// Set read timeout
    pub fn set_read_timeout(&mut self, timeout: Option<Duration>) -> Result<(), TransportError> {
        self.socket.set_read_timeout(timeout)?;
        Ok(())
    }
    
    /// Set write timeout
    pub fn set_write_timeout(&mut self, timeout: Option<Duration>) -> Result<(), TransportError> {
        self.socket.set_write_timeout(timeout)?;
        Ok(())
    }
}

/// Reliable UDP transport with retransmissions
pub struct ReliableUdpTransport {
    udp: UdpTransport,
    window_size: usize,
    send_seq: u32,
    recv_seq: u32,
    in_flight: std::collections::HashMap<u32, (Instant, SocketAddr, Bytes)>,
    rtt_estimate: Duration,
    rtt_variance: Duration,
}

impl ReliableUdpTransport {
    /// Create a new reliable UDP transport
    pub fn bind(local_addr: SocketAddr) -> Result<Self, TransportError> {
        Ok(Self {
            udp: UdpTransport::bind(local_addr)?,
            window_size: 32,
            send_seq: 0,
            recv_seq: 0,
            in_flight: std::collections::HashMap::new(),
            rtt_estimate: Duration::from_millis(100),
            rtt_variance: Duration::from_millis(50),
        })
    }
    
    /// Send a reliable packet
    pub fn send_to(&mut self, data: &[u8], dest: SocketAddr) -> Result<(), TransportError> {
        let seq = self.send_seq;
        self.send_seq += 1;
        
        // Serialize packet with sequence number
        let mut packet = Vec::new();
        packet.extend_from_slice(&seq.to_be_bytes());
        packet.extend_from_slice(data);
        
        self.udp.send_to(&packet, dest)?;
        
        self.in_flight.insert(seq, (Instant::now(), dest, Bytes::copy_from_slice(&packet)));
        
        Ok(())
    }
    
    /// Acknowledgments handling
    pub fn poll(&mut self) -> Result<Vec<(SocketAddr, Bytes)>, TransportError> {
        // Poll underlying transport
        self.udp.poll()?;
        
        let mut received = Vec::new();
        
        while let Some((addr, data)) = self.udp.next_packet() {
            // Check if it's an acknowledgment
            if data.len() >= 8 && &data[0..4] == b"ACK:" {
                let seq_bytes = <[u8; 4]>::try_from(&data[4..8]).unwrap();
                let seq = u32::from_be_bytes(seq_bytes);
                self.handle_ack(seq);
            } else if data.len() >= 4 {
                // Check if it's a data packet
                let seq_bytes = <[u8; 4]>::try_from(&data[0..4]).unwrap();
                let seq = u32::from_be_bytes(seq_bytes);
                self.recv_seq = seq + 1;
                
                // Send acknowledgment
                let mut ack = Vec::new();
                ack.extend_from_slice(b"ACK:");
                ack.extend_from_slice(&seq.to_be_bytes());
                self.udp.send_to(&ack, addr)?;
                
                // Deliver payload
                if data.len() > 4 {
                    received.push((addr, data.slice(4..)));
                }
            }
        }
        
        // Retransmit packets
        self.retransmit()?;
        
        Ok(received)
    }
    
    /// Handle acknowledgment
    fn handle_ack(&mut self, seq: u32) {
        if let Some((sent_time, _, _)) = self.in_flight.remove(&seq) {
            let rtt = sent_time.elapsed();
            
            // Update RTT estimate
            let sample = rtt.as_millis() as i64;
            let rtt_est = self.rtt_estimate.as_millis() as i64;
            let rtt_var = self.rtt_variance.as_millis() as i64;
            let var = (rtt_est - sample).abs();
            
            self.rtt_variance = Duration::from_millis((rtt_var * 3 / 4 + var / 4) as u64);
            self.rtt_estimate = Duration::from_millis((rtt_est * 7 / 8 + sample / 8) as u64);
        }
    }
    
    /// Retransmit unacknowledged packets
    fn retransmit(&mut self) -> Result<(), TransportError> {
        let now = Instant::now();
        let timeout = self.rtt_estimate * 2 + self.rtt_variance * 4;
        
        let to_retransmit: Vec<_> = self.in_flight.iter()
            .filter(|(_, (time, _, _))| now.duration_since(*time) > timeout)
            .map(|(&seq, (_, addr, data))| (seq, *addr, data.clone()))
            .collect();
        
        for (_, addr, data) in to_retransmit {
            self.udp.send_to(&data, addr)?;
        }
        
        Ok(())
    }
    
    /// Get local address
    pub fn local_addr(&self) -> Result<SocketAddr, TransportError> {
        self.udp.local_addr()
    }
}

#[derive(Debug, Error)]
pub enum TransportError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Packet too large")]
    PacketTooLarge,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, SocketAddrV4};

    #[test]
    fn test_udp_transport_creation() {
        let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0));
        let transport = UdpTransport::bind(addr).unwrap();
        
        assert!(transport.local_addr().is_ok());
    }

    #[test]
    fn test_reliable_udp_transport() {
        let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0));
        let transport = ReliableUdpTransport::bind(addr).unwrap();
        
        assert!(transport.local_addr().is_ok());
    }
}
