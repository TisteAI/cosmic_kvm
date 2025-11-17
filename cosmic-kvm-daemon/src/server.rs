//! Server implementation

use crate::config::Config;
use crate::discovery::Discovery;
use anyhow::Result;
use tokio::net::TcpListener;

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
        let addr = format!("0.0.0.0:{}", self.port);
        let listener = TcpListener::bind(&addr).await?;
        tracing::info!("Listening on {}", addr);

        loop {
            match listener.accept().await {
                Ok((_socket, addr)) => {
                    tracing::info!("New connection from {}", addr);
                    // TODO: Handle connection
                    // - TLS handshake
                    // - Protocol handshake
                    // - Input event loop
                }
                Err(e) => {
                    tracing::error!("Failed to accept connection: {}", e);
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
