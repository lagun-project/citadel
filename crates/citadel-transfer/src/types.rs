//! Core types for Citadel Transfer protocol

use serde::{Deserialize, Serialize};

/// Unique identifier for a logical stream of data
pub type StreamId = u128;

/// Configuration epoch/generation number
pub type Epoch = u32;

/// Sequence number for packet ordering
pub type SeqNo = u64;

/// Node identifier
pub type NodeId = u64;

/// Default MTU for payload data (optimal for most networks)
pub const DEFAULT_PAYLOAD_MTU: usize = 1200;

/// Message type classification
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MsgKind {
    /// Regular data packet
    Data,
    /// Acknowledgment packet
    Ack,
    /// Control message (flow control, etc.)
    Control,
    /// BFT consensus message
    Bft,
    /// TGP commit message (bilateral coordination complete)
    TgpCommit,
}

/// Packet header containing routing and sequencing information
///
/// Total size: 48 bytes
/// - stream_id: 16 bytes (u128)
/// - epoch: 4 bytes (u32)
/// - seq: 8 bytes (u64)
/// - kind: 1 byte (enum)
/// - flags: 1 byte
/// - body_len: 2 bytes (u16)
/// - padding: 16 bytes (alignment)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PacketHeader {
    /// Stream this packet belongs to
    pub stream_id: StreamId,
    /// Configuration epoch
    pub epoch: Epoch,
    /// Sequence number within stream
    pub seq: SeqNo,
    /// Message type
    pub kind: MsgKind,
    /// Control flags
    pub flags: u8,
    /// Length of packet body
    pub body_len: u16,
}

impl PacketHeader {
    /// Create a new data packet header
    pub fn new_data(stream_id: StreamId, epoch: Epoch, seq: SeqNo, body_len: u16) -> Self {
        Self {
            stream_id,
            epoch,
            seq,
            kind: MsgKind::Data,
            flags: 0,
            body_len,
        }
    }

    /// Create a new control packet header
    pub fn new_control(stream_id: StreamId, epoch: Epoch, seq: SeqNo) -> Self {
        Self {
            stream_id,
            epoch,
            seq,
            kind: MsgKind::Control,
            flags: 0,
            body_len: 0,
        }
    }
}

/// Complete packet with header and body
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Packet {
    /// Packet header
    pub hdr: PacketHeader,
    /// Packet body/payload
    pub body: bytes::Bytes,
}

impl Packet {
    /// Create a new packet with the given header and body
    pub fn new(hdr: PacketHeader, body: bytes::Bytes) -> Self {
        Self { hdr, body }
    }

    /// Create a data packet
    pub fn data(stream_id: StreamId, epoch: Epoch, seq: SeqNo, body: bytes::Bytes) -> Self {
        let hdr = PacketHeader::new_data(stream_id, epoch, seq, body.len() as u16);
        Self { hdr, body }
    }

    /// Get the stream ID
    pub fn stream_id(&self) -> StreamId {
        self.hdr.stream_id
    }

    /// Get the sequence number
    pub fn seq(&self) -> SeqNo {
        self.hdr.seq
    }

    /// Check if this is a data packet
    pub fn is_data(&self) -> bool {
        self.hdr.kind == MsgKind::Data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_creation() {
        let body = bytes::Bytes::from_static(b"hello world");
        let packet = Packet::data(12345, 1, 0, body.clone());

        assert_eq!(packet.stream_id(), 12345);
        assert_eq!(packet.seq(), 0);
        assert!(packet.is_data());
        assert_eq!(packet.body, body);
    }

    #[test]
    fn test_header_creation() {
        let hdr = PacketHeader::new_data(999, 2, 42, 100);
        assert_eq!(hdr.stream_id, 999);
        assert_eq!(hdr.epoch, 2);
        assert_eq!(hdr.seq, 42);
        assert_eq!(hdr.body_len, 100);
        assert_eq!(hdr.kind, MsgKind::Data);
    }
}
