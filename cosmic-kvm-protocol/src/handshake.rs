//! Handshake and authentication protocol

use serde::{Deserialize, Serialize};

/// Handshake messages for connection establishment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HandshakeMessage {
    /// Initial hello from client
    Hello(HelloMessage),
    /// Server's response with authentication challenge
    Challenge(ChallengeMessage),
    /// Client's authentication response
    Response(ResponseMessage),
    /// Server accepts connection
    Accept(AcceptMessage),
    /// Server rejects connection
    Reject(RejectMessage),
}

/// Initial hello message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelloMessage {
    /// Protocol version the client supports
    pub protocol_version: String,
    /// Client's device name
    pub device_name: String,
    /// Client's unique device ID
    pub device_id: String,
    /// Capabilities the client supports
    pub capabilities: Capabilities,
}

/// Device capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capabilities {
    /// Can send keyboard events
    pub keyboard: bool,
    /// Can send mouse events
    pub mouse: bool,
    /// Can share clipboard
    pub clipboard: bool,
    /// Supported clipboard MIME types
    pub clipboard_types: Vec<String>,
}

impl Default for Capabilities {
    fn default() -> Self {
        Self {
            keyboard: true,
            mouse: true,
            clipboard: true,
            clipboard_types: vec![
                "text/plain".to_string(),
                "text/plain;charset=utf-8".to_string(),
            ],
        }
    }
}

/// Authentication challenge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeMessage {
    /// Random nonce for challenge-response
    pub nonce: Vec<u8>,
    /// Server's TLS certificate fingerprint
    pub cert_fingerprint: String,
}

/// Authentication response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMessage {
    /// Response to challenge (signed nonce)
    pub signature: Vec<u8>,
    /// Client's TLS certificate fingerprint
    pub cert_fingerprint: String,
}

/// Connection accepted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptMessage {
    /// Server's device name
    pub device_name: String,
    /// Server's capabilities
    pub capabilities: Capabilities,
    /// Assigned session ID
    pub session_id: String,
}

/// Connection rejected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RejectMessage {
    /// Reason for rejection
    pub reason: String,
}
