//! Error types for the protocol

use thiserror::Error;

pub type Result<T> = std::result::Result<T, ProtocolError>;

#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("Invalid protocol magic bytes")]
    InvalidMagic,

    #[error("Incomplete message received")]
    IncompleteMessage,

    #[error("Message too large: {0} bytes")]
    MessageTooLarge(usize),

    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Protocol version mismatch: expected {expected}, got {actual}")]
    VersionMismatch { expected: String, actual: String },

    #[error("Unsupported feature: {0}")]
    UnsupportedFeature(String),
}
