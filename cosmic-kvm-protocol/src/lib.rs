//! Network protocol for COSMIC KVM sharing
//!
//! This crate defines the binary protocol used for communication between
//! COSMIC KVM instances. It supports keyboard, mouse, and clipboard sharing.

use serde::{Deserialize, Serialize};

pub mod error;
pub mod events;
pub mod handshake;

pub use error::{ProtocolError, Result};
pub use events::*;
pub use handshake::*;

/// Protocol version following semantic versioning
pub const PROTOCOL_VERSION: &str = "0.1.0";

/// Magic bytes to identify COSMIC KVM protocol
pub const PROTOCOL_MAGIC: [u8; 4] = *b"CKVT";

/// Maximum message size (16 MB) to prevent memory exhaustion attacks
pub const MAX_MESSAGE_SIZE: usize = 16 * 1024 * 1024;

/// Message envelope containing protocol version and payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Protocol version for compatibility checking
    pub version: String,
    /// The actual message payload
    pub payload: Payload,
}

/// All possible message payloads
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Payload {
    /// Handshake and authentication messages
    Handshake(HandshakeMessage),
    /// Input events (keyboard, mouse, etc.)
    Input(InputEvent),
    /// Clipboard synchronization
    Clipboard(ClipboardData),
    /// Display information exchange
    Display(DisplayInfo),
    /// Control messages
    Control(ControlMessage),
    /// Heartbeat to maintain connection
    Heartbeat,
}

impl Message {
    /// Create a new message with current protocol version
    pub fn new(payload: Payload) -> Self {
        Self {
            version: PROTOCOL_VERSION.to_string(),
            payload,
        }
    }

    /// Serialize message to bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut bytes = PROTOCOL_MAGIC.to_vec();
        let encoded = bincode::serialize(self)?;

        // Check message size
        if encoded.len() > MAX_MESSAGE_SIZE {
            return Err(ProtocolError::MessageTooLarge(encoded.len()));
        }

        // Length prefix (4 bytes, big endian)
        bytes.extend_from_slice(&(encoded.len() as u32).to_be_bytes());
        bytes.extend_from_slice(&encoded);

        Ok(bytes)
    }

    /// Deserialize message from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        // Verify magic bytes
        if bytes.len() < 8 || &bytes[0..4] != &PROTOCOL_MAGIC {
            return Err(ProtocolError::InvalidMagic);
        }

        // Read length
        let length = u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]) as usize;

        // Verify length
        if length > MAX_MESSAGE_SIZE {
            return Err(ProtocolError::MessageTooLarge(length));
        }

        if bytes.len() < 8 + length {
            return Err(ProtocolError::IncompleteMessage);
        }

        // Deserialize
        let message: Self = bincode::deserialize(&bytes[8..8 + length])?;

        Ok(message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_serialization() {
        let msg = Message::new(Payload::Heartbeat);
        let bytes = msg.to_bytes().unwrap();
        let decoded = Message::from_bytes(&bytes).unwrap();

        assert_eq!(msg.version, decoded.version);
        matches!(decoded.payload, Payload::Heartbeat);
    }

    #[test]
    fn test_message_size_limit() {
        // Create a large clipboard message
        let large_data = vec![0u8; MAX_MESSAGE_SIZE + 1];
        let msg = Message::new(Payload::Clipboard(ClipboardData {
            mime_type: "text/plain".to_string(),
            data: large_data,
        }));

        assert!(msg.to_bytes().is_err());
    }
}
