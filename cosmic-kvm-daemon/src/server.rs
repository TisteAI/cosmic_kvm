//! Server implementation
//!
//! Captures local input and forwards it to connected clients

use crate::capture::InputCapture;
use crate::config::Config;
use crate::discovery::Discovery;
use anyhow::{Context, Result};
use cosmic_kvm_protocol::{
    AcceptMessage, Capabilities, ChallengeMessage, HandshakeMessage, HelloMessage, Message,
    Payload, RejectMessage,
};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};

pub struct Server {
    config: Config,
    port: u16,
    discovery: Option<Discovery>,
}

impl Server {
    pub async fn new(config: Config, port: u16) -> Result<Self> {
        let discovery = if config.enable_mdns {
            let disc = Discovery::new()?;
            disc.advertise(&config.device_name, port, &config.device_id)?;
            Some(disc)
        } else {
            None
        };

        Ok(Self {
            config,
            port,
            discovery,
        })
    }

    pub async fn run(self) -> Result<()> {
        info!("Starting KVM server on port {}", self.port);

        // Start input capture
        let (mut capture, event_receiver) = InputCapture::new()?;

        info!("Auto-detecting input devices...");
        capture.add_all_devices()?;

        let event_receiver = Arc::new(event_receiver);

        // Spawn capture task
        let _capture_handle = tokio::spawn(async move {
            if let Err(e) = capture.run().await {
                error!("Input capture failed: {}", e);
            }
        });

        // Start accepting client connections
        let addr = format!("0.0.0.0:{}", self.port);
        let listener = TcpListener::bind(&addr)
            .await
            .with_context(|| format!("Failed to bind to {}", addr))?;

        info!("Server listening on {}", addr);
        info!("Clients can connect and will receive input events");

        let config = Arc::new(self.config.clone());

        loop {
            match listener.accept().await {
                Ok((socket, addr)) => {
                    info!("New connection from {}", addr);

                    let config = Arc::clone(&config);
                    let event_receiver_clone = Arc::clone(&event_receiver);

                    // Spawn handler for this client
                    tokio::spawn(async move {
                        // Create new receiver from the sender
                        let mut client_receiver = event_receiver_clone.resubscribe();

                        if let Err(e) =
                            handle_client(socket, config, &mut client_receiver).await
                        {
                            error!("Client {} error: {}", addr, e);
                        } else {
                            info!("Client {} disconnected", addr);
                        }
                    });
                }
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                }
            }
        }
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        if let Some(ref discovery) = self.discovery {
            let _ = discovery.shutdown();
        }
    }
}

async fn handle_client(
    stream: TcpStream,
    config: Arc<Config>,
    event_receiver: &mut broadcast::Receiver<cosmic_kvm_protocol::InputEvent>,
) -> Result<()> {
    let peer_addr = stream.peer_addr()?;

    // For now, use plain TCP (no TLS)
    let mut connection = PlainConnection { stream };

    // Perform handshake
    let client_info = match handshake(&mut connection, &config).await {
        Ok(info) => {
            info!("Client {} authenticated: {}", peer_addr, info.device_name);
            info
        }
        Err(e) => {
            warn!("Client {} handshake failed: {}", peer_addr, e);
            return Err(e);
        }
    };

    // Send events to client
    let mut event_count = 0u64;
    let mut heartbeat_interval = tokio::time::interval(std::time::Duration::from_secs(30));

    loop {
        tokio::select! {
            // Forward input events
            event = event_receiver.recv() => {
                match event {
                    Ok(input_event) => {
                        event_count += 1;

                        let message = Message::new(Payload::Input(input_event));
                        if let Err(e) = connection.send(&message).await {
                            error!("Failed to send event to {}: {}", peer_addr, e);
                            break;
                        }

                        if event_count % 1000 == 0 {
                            debug!("Sent {} events to {}", event_count, peer_addr);
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(skipped)) => {
                        warn!("Client {} lagged, skipped {} events", peer_addr, skipped);
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        info!("Input capture stopped");
                        break;
                    }
                }
            }

            // Send periodic heartbeat
            _ = heartbeat_interval.tick() => {
                let heartbeat = Message::new(Payload::Heartbeat);
                if let Err(e) = connection.send(&heartbeat).await {
                    error!("Failed to send heartbeat to {}: {}", peer_addr, e);
                    break;
                }
                debug!("Sent heartbeat to {}", peer_addr);
            }
        }
    }

    info!(
        "Client {} session ended. Sent {} events",
        client_info.device_name, event_count
    );

    Ok(())
}

async fn handshake(
    connection: &mut PlainConnection,
    config: &Config,
) -> Result<HelloMessage> {
    // Receive Hello
    let message = connection.receive().await?;
    let hello = match message.payload {
        Payload::Handshake(HandshakeMessage::Hello(h)) => h,
        _ => {
            let reject = Message::new(Payload::Handshake(HandshakeMessage::Reject(
                RejectMessage {
                    reason: "Expected Hello message".to_string(),
                },
            )));
            let _ = connection.send(&reject).await;
            anyhow::bail!("Expected Hello message");
        }
    };

    debug!(
        "Received Hello from {} ({})",
        hello.device_name, hello.device_id
    );

    // Check protocol version compatibility
    if hello.protocol_version != cosmic_kvm_protocol::PROTOCOL_VERSION {
        let reject = Message::new(Payload::Handshake(HandshakeMessage::Reject(
            RejectMessage {
                reason: format!(
                    "Protocol version mismatch: expected {}, got {}",
                    cosmic_kvm_protocol::PROTOCOL_VERSION,
                    hello.protocol_version
                ),
            },
        )));
        connection.send(&reject).await?;
        anyhow::bail!("Protocol version mismatch");
    }

    // Send Challenge
    let challenge = Message::new(Payload::Handshake(HandshakeMessage::Challenge(
        ChallengeMessage {
            nonce: vec![1, 2, 3, 4], // Simple nonce for now
            cert_fingerprint: "server-fingerprint".to_string(),
        },
    )));

    connection.send(&challenge).await?;
    debug!("Sent Challenge");

    // Receive Response
    let response = connection.receive().await?;
    let _response_data = match response.payload {
        Payload::Handshake(HandshakeMessage::Response(r)) => r,
        _ => {
            let reject = Message::new(Payload::Handshake(HandshakeMessage::Reject(
                RejectMessage {
                    reason: "Expected Response message".to_string(),
                },
            )));
            let _ = connection.send(&reject).await;
            anyhow::bail!("Expected Response message");
        }
    };

    debug!("Received Response");

    // For now, accept all clients (no actual verification)
    // TODO: Implement proper certificate verification
    let accept = Message::new(Payload::Handshake(HandshakeMessage::Accept(
        AcceptMessage {
            device_name: config.device_name.clone(),
            capabilities: Capabilities::default(),
            session_id: format!("session-{}", uuid::Uuid::new_v4()),
        },
    )));

    connection.send(&accept).await?;
    debug!("Sent Accept");

    Ok(hello)
}

/// Plain TCP connection (no TLS for now)
struct PlainConnection {
    stream: TcpStream,
}

impl PlainConnection {
    async fn send(&mut self, message: &Message) -> Result<()> {
        let bytes = message.to_bytes()?;
        self.stream.write_all(&bytes).await?;
        self.stream.flush().await?;
        Ok(())
    }

    async fn receive(&mut self) -> Result<Message> {
        // Read magic bytes and length
        let mut header = [0u8; 8];
        self.stream.read_exact(&mut header).await?;

        let length = u32::from_be_bytes([header[4], header[5], header[6], header[7]]) as usize;

        // Validate length
        if length > cosmic_kvm_protocol::MAX_MESSAGE_SIZE {
            anyhow::bail!("Message too large: {} bytes", length);
        }

        // Read message body
        let mut body = vec![0u8; length];
        self.stream.read_exact(&mut body).await?;

        // Reconstruct full message
        let mut full_message = header.to_vec();
        full_message.extend_from_slice(&body);

        Ok(Message::from_bytes(&full_message)?)
    }
}
