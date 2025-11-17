//! mDNS service discovery
//!
//! Advertises and discovers COSMIC KVM services on the local network

use anyhow::Result;
use mdns_sd::{ServiceDaemon, ServiceInfo};
use std::collections::HashMap;

const SERVICE_TYPE: &str = "_cosmic-kvm._tcp.local.";

pub struct Discovery {
    daemon: ServiceDaemon,
}

impl Discovery {
    /// Create a new discovery service
    pub fn new() -> Result<Self> {
        let daemon = ServiceDaemon::new()?;
        Ok(Self { daemon })
    }

    /// Advertise this device's KVM service
    pub fn advertise(&self, device_name: &str, port: u16, device_id: &str) -> Result<()> {
        let hostname = hostname::get()?;
        let hostname_str = hostname.to_string_lossy();

        let mut properties = HashMap::new();
        properties.insert("device_id".to_string(), device_id.to_string());

        let service_info = ServiceInfo::new(
            SERVICE_TYPE,
            device_name,
            &hostname_str,
            (), // Default IP addresses
            port,
            Some(properties),
        )?;

        self.daemon.register(service_info)?;
        tracing::info!("Advertising service: {} on port {}", device_name, port);

        Ok(())
    }

    /// Browse for available KVM services
    pub fn browse(&self) -> Result<()> {
        let receiver = self.daemon.browse(SERVICE_TYPE)?;

        tokio::spawn(async move {
            while let Ok(event) = receiver.recv_async().await {
                match event {
                    mdns_sd::ServiceEvent::ServiceResolved(info) => {
                        tracing::info!(
                            "Discovered service: {} at {:?}:{}",
                            info.get_fullname(),
                            info.get_addresses(),
                            info.get_port()
                        );
                    }
                    mdns_sd::ServiceEvent::ServiceRemoved(_, fullname) => {
                        tracing::info!("Service removed: {}", fullname);
                    }
                    _ => {}
                }
            }
        });

        Ok(())
    }

    /// Stop advertising
    pub fn shutdown(&self) -> Result<()> {
        self.daemon.shutdown()?;
        Ok(())
    }
}
