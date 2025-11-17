//! Configuration management

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

use super::Mode;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Device name to advertise
    pub device_name: String,

    /// Unique device ID (generated if not set)
    pub device_id: String,

    /// Default operating mode
    #[serde(default = "default_mode")]
    pub default_mode: Mode,

    /// Default server to connect to (client mode)
    pub default_server: Option<String>,

    /// Enable mDNS service discovery
    #[serde(default = "default_true")]
    pub enable_mdns: bool,

    /// Trusted device fingerprints
    #[serde(default)]
    pub trusted_devices: Vec<TrustedDevice>,

    /// Input backend preference
    #[serde(default)]
    pub input_backend: InputBackendConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustedDevice {
    pub name: String,
    pub device_id: String,
    pub fingerprint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputBackendConfig {
    Auto,
    UInput,
    Wayland,
}

impl Default for InputBackendConfig {
    fn default() -> Self {
        Self::Auto
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            device_name: hostname::get()
                .ok()
                .and_then(|h| h.into_string().ok())
                .unwrap_or_else(|| "cosmic-kvm".to_string()),
            device_id: uuid::Uuid::new_v4().to_string(),
            default_mode: default_mode(),
            default_server: None,
            enable_mdns: true,
            trusted_devices: Vec::new(),
            input_backend: InputBackendConfig::Auto,
        }
    }
}

impl Config {
    /// Load configuration from file or create default
    pub fn load(path: Option<&Path>) -> Result<Self> {
        let config_path = path
            .map(|p| p.to_path_buf())
            .or_else(|| {
                dirs::config_dir().map(|mut p| {
                    p.push("cosmic-kvm");
                    p.push("config.toml");
                    p
                })
            })
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;

        if config_path.exists() {
            tracing::info!("Loading configuration from {:?}", config_path);
            let contents = std::fs::read_to_string(&config_path)?;
            let config: Config = toml::from_str(&contents)?;
            Ok(config)
        } else {
            tracing::info!("Creating default configuration at {:?}", config_path);
            let config = Config::default();

            // Create config directory if it doesn't exist
            if let Some(parent) = config_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            // Save default config
            let contents = toml::to_string_pretty(&config)?;
            std::fs::write(&config_path, contents)?;

            Ok(config)
        }
    }

    /// Save configuration to file
    pub fn save(&self, path: Option<&Path>) -> Result<()> {
        let config_path = path
            .map(|p| p.to_path_buf())
            .or_else(|| {
                dirs::config_dir().map(|mut p| {
                    p.push("cosmic-kvm");
                    p.push("config.toml");
                    p
                })
            })
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;

        let contents = toml::to_string_pretty(self)?;
        std::fs::write(&config_path, contents)?;

        Ok(())
    }
}

fn default_mode() -> Mode {
    Mode::Server
}

fn default_true() -> bool {
    true
}
