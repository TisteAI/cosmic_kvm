//! COSMIC KVM Daemon
//!
//! Background service that handles:
//! - Network communication with other machines
//! - Input event capture and injection
//! - Service discovery via mDNS
//! - D-Bus interface for UI control

mod config;
mod discovery;
mod network;
mod server;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[derive(Parser, Debug)]
#[command(name = "cosmic-kvm-daemon")]
#[command(about = "COSMIC KVM sharing daemon", long_about = None)]
struct Args {
    /// Configuration file path
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Run in server mode (accept connections)
    #[arg(short, long)]
    server: bool,

    /// Run in client mode (connect to server)
    #[arg(short = 'C', long)]
    client: bool,

    /// Server address to connect to (client mode)
    #[arg(long)]
    connect: Option<String>,

    /// Port to listen on (default: 24900)
    #[arg(short, long, default_value = "24900")]
    port: u16,

    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    let level = if args.debug { Level::DEBUG } else { Level::INFO };
    let subscriber = FmtSubscriber::builder().with_max_level(level).finish();
    tracing::subscriber::set_global_default(subscriber)?;

    tracing::info!("Starting COSMIC KVM daemon v{}", env!("CARGO_PKG_VERSION"));

    // Load configuration
    let config = config::Config::load(args.config.as_deref())?;
    tracing::debug!("Loaded configuration: {:?}", config);

    // Determine mode
    let mode = if args.server {
        Mode::Server
    } else if args.client {
        Mode::Client
    } else {
        config.default_mode
    };

    match mode {
        Mode::Server => {
            tracing::info!("Starting in server mode on port {}", args.port);
            let server = server::Server::new(config, args.port).await?;
            server.run().await?;
        }
        Mode::Client => {
            if let Some(addr) = args.connect.or(config.default_server) {
                tracing::info!("Starting in client mode, connecting to {}", addr);
                // TODO: Implement client mode
                anyhow::bail!("Client mode not yet implemented");
            } else {
                anyhow::bail!("Client mode requires --connect address");
            }
        }
    }

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
enum Mode {
    Server,
    Client,
}
