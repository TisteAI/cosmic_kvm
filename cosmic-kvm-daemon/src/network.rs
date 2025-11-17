//! Network communication layer
//!
//! Handles TLS-encrypted communication between KVM instances

use anyhow::Result;
use cosmic_kvm_protocol::Message;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_rustls::TlsStream;

pub struct Connection {
    stream: TlsStream<TcpStream>,
}

impl Connection {
    /// Send a message
    pub async fn send(&mut self, message: &Message) -> Result<()> {
        let bytes = message.to_bytes()?;
        self.stream.write_all(&bytes).await?;
        self.stream.flush().await?;
        Ok(())
    }

    /// Receive a message
    pub async fn receive(&mut self) -> Result<Message> {
        // Read magic bytes and length
        let mut header = [0u8; 8];
        self.stream.read_exact(&mut header).await?;

        let length = u32::from_be_bytes([header[4], header[5], header[6], header[7]]) as usize;

        // Read message body
        let mut body = vec![0u8; length];
        self.stream.read_exact(&mut body).await?;

        // Reconstruct full message
        let mut full_message = header.to_vec();
        full_message.extend_from_slice(&body);

        Ok(Message::from_bytes(&full_message)?)
    }
}
