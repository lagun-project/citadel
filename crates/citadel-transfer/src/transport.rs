//! UDP transport layer for high-performance packet delivery
//!
//! Provides a thin wrapper around tokio's UdpSocket with:
//! - Configurable send/receive buffer sizes
//! - Batched packet processing
//! - Handler registration for incoming packets

use std::net::SocketAddr;
use std::sync::Arc;

use socket2::{Domain, Protocol, Socket, Type};
use tokio::net::UdpSocket;

use crate::types::Packet;

/// Transport configuration
#[derive(Debug, Clone)]
pub struct TransportConfig {
    /// Address to bind to
    pub bind: SocketAddr,
    /// Batch size for packet processing
    pub batch: usize,
    /// Send buffer size in bytes
    pub sndbuf: usize,
    /// Receive buffer size in bytes
    pub rcvbuf: usize,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            bind: "0.0.0.0:9000".parse().unwrap(),
            batch: 64,
            sndbuf: 4 * 1024 * 1024, // 4MB
            rcvbuf: 4 * 1024 * 1024, // 4MB
        }
    }
}

/// Handler function for incoming packets
pub type Handler = Box<dyn Fn(SocketAddr, Packet) + Send + Sync>;

/// Internal state for handlers
pub(crate) struct HandlerState {
    pub handler: std::sync::Mutex<Option<Handler>>,
}

/// Internal UDP transport implementation
pub(crate) struct UdpTransport {
    pub socket: Arc<UdpSocket>,
    pub handlers: Arc<HandlerState>,
}

impl UdpTransport {
    /// Bind to the given address with configured buffer sizes
    pub async fn bind(addr: SocketAddr, sndbuf: usize, rcvbuf: usize) -> anyhow::Result<Self> {
        // Create socket with socket2 for buffer configuration
        let domain = if addr.is_ipv4() {
            Domain::IPV4
        } else {
            Domain::IPV6
        };

        let socket = Socket::new(domain, Type::DGRAM, Some(Protocol::UDP))?;

        // Set buffer sizes before binding
        socket.set_send_buffer_size(sndbuf)?;
        socket.set_recv_buffer_size(rcvbuf)?;

        // Allow address reuse
        socket.set_reuse_address(true)?;

        // Bind
        socket.bind(&addr.into())?;
        socket.set_nonblocking(true)?;

        // Convert to tokio socket
        let std_socket: std::net::UdpSocket = socket.into();
        let tokio_socket = UdpSocket::from_std(std_socket)?;

        tracing::info!(
            "UDP transport bound to {} (sndbuf={}, rcvbuf={})",
            addr,
            sndbuf,
            rcvbuf
        );

        Ok(Self {
            socket: Arc::new(tokio_socket),
            handlers: Arc::new(HandlerState {
                handler: std::sync::Mutex::new(None),
            }),
        })
    }
}

/// High-level transport handle for sending and receiving packets
pub struct TransportHandle {
    inner: Arc<UdpTransport>,
}

impl TransportHandle {
    /// Create a new transport handle with the given configuration
    pub async fn new(cfg: TransportConfig) -> anyhow::Result<Self> {
        let transport = UdpTransport::bind(cfg.bind, cfg.sndbuf, cfg.rcvbuf).await?;
        Ok(Self {
            inner: Arc::new(transport),
        })
    }

    /// Send a packet to the given address
    pub async fn send(&self, addr: SocketAddr, pkt: Packet) -> anyhow::Result<()> {
        // Simple framing: length-prefix the body
        let mut buf = Vec::with_capacity(2 + pkt.body.len());
        buf.extend_from_slice(&(pkt.body.len() as u16).to_le_bytes());
        buf.extend_from_slice(&pkt.body);

        self.inner.socket.send_to(&buf, addr).await?;
        Ok(())
    }

    /// Send raw bytes to the given address (no framing)
    pub async fn send_raw(&self, addr: SocketAddr, data: &[u8]) -> anyhow::Result<()> {
        self.inner.socket.send_to(data, addr).await?;
        Ok(())
    }

    /// Receive raw bytes from any sender
    pub async fn recv_raw(&self, buf: &mut [u8]) -> anyhow::Result<(usize, SocketAddr)> {
        let (len, addr) = self.inner.socket.recv_from(buf).await?;
        Ok((len, addr))
    }

    /// Register a handler for incoming packets
    pub fn register_handler(&self, h: Handler) {
        *self.inner.handlers.handler.lock().unwrap() = Some(h);
    }

    /// Get the underlying socket (for advanced use cases)
    pub fn socket(&self) -> Arc<UdpSocket> {
        self.inner.socket.clone()
    }

    /// Get the local address this transport is bound to
    pub fn local_addr(&self) -> anyhow::Result<SocketAddr> {
        Ok(self.inner.socket.local_addr()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_transport_creation() {
        let config = TransportConfig {
            bind: "127.0.0.1:0".parse().unwrap(),
            ..Default::default()
        };

        let transport = TransportHandle::new(config).await.unwrap();
        let addr = transport.local_addr().unwrap();
        assert!(addr.port() > 0);
    }

    #[tokio::test]
    async fn test_send_recv() {
        // Create two transports
        let cfg1 = TransportConfig {
            bind: "127.0.0.1:0".parse().unwrap(),
            ..Default::default()
        };
        let cfg2 = TransportConfig {
            bind: "127.0.0.1:0".parse().unwrap(),
            ..Default::default()
        };

        let t1 = TransportHandle::new(cfg1).await.unwrap();
        let t2 = TransportHandle::new(cfg2).await.unwrap();

        let addr2 = t2.local_addr().unwrap();

        // Send from t1 to t2
        t1.send_raw(addr2, b"hello").await.unwrap();

        // Receive on t2
        let mut buf = vec![0u8; 1024];
        let (len, from_addr) = t2.recv_raw(&mut buf).await.unwrap();

        assert_eq!(&buf[..len], b"hello");
        assert_eq!(from_addr, t1.local_addr().unwrap());
    }
}
