//! Input event capture from local devices
//!
//! Captures keyboard and mouse events from /dev/input/event* devices
//! and converts them to protocol events for network transmission.

use anyhow::{Context, Result};
use cosmic_kvm_protocol::{
    InputEvent, KeyboardEvent, ModifierState, MouseButtonEvent, MouseMoveEvent, MouseMovement,
    MouseWheelEvent,
};
use evdev::{Device, EventType, InputEventKind, Key};
use std::path::Path;
use tokio::sync::broadcast;
use tracing::{debug, info, warn};

/// Input capture manager
pub struct InputCapture {
    devices: Vec<Device>,
    sender: broadcast::Sender<InputEvent>,
}

impl InputCapture {
    /// Create a new input capture manager
    pub fn new() -> Result<(Self, broadcast::Receiver<InputEvent>)> {
        let (sender, receiver) = broadcast::channel(1024);

        let capture = Self {
            devices: Vec::new(),
            sender,
        };

        Ok((capture, receiver))
    }

    /// Add a device to capture from
    pub fn add_device<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let device = Device::open(path.as_ref())
            .with_context(|| format!("Failed to open device: {:?}", path.as_ref()))?;

        info!(
            "Added capture device: {} ({})",
            device.name().unwrap_or("unknown"),
            path.as_ref().display()
        );

        self.devices.push(device);
        Ok(())
    }

    /// Auto-detect and add all keyboard and mouse devices
    pub fn add_all_devices(&mut self) -> Result<()> {
        let input_dir = Path::new("/dev/input");

        if !input_dir.exists() {
            anyhow::bail!("/dev/input directory not found");
        }

        // Scan for eventN devices
        for entry in std::fs::read_dir(input_dir)? {
            let entry = entry?;
            let path = entry.path();

            // Only process eventN files
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with("event") {
                    // Try to open device to check if it's readable
                    match Device::open(&path) {
                        Ok(device) => {
                            // Check if device is keyboard or mouse
                            if is_keyboard_or_mouse(&device) {
                                info!(
                                    "Auto-detected device: {} at {}",
                                    device.name().unwrap_or("unknown"),
                                    path.display()
                                );
                                self.devices.push(device);
                            }
                        }
                        Err(e) => {
                            debug!("Skipping {}: {}", path.display(), e);
                        }
                    }
                }
            }
        }

        if self.devices.is_empty() {
            warn!("No input devices found. Make sure you have permission to access /dev/input/event*");
        } else {
            info!("Added {} input devices", self.devices.len());
        }

        Ok(())
    }

    /// Start capturing events
    pub async fn run(self) -> Result<()> {
        if self.devices.is_empty() {
            anyhow::bail!("No devices to capture from. Add devices first.");
        }

        info!("Starting input capture from {} devices", self.devices.len());

        // Create async tasks for each device
        let mut handles = Vec::new();

        for device in self.devices {
            let sender = self.sender.clone();
            let handle = tokio::task::spawn_blocking(move || {
                Self::capture_device(device, sender)
            });
            handles.push(handle);
        }

        // Wait for all capture tasks
        for handle in handles {
            if let Err(e) = handle.await {
                warn!("Capture task failed: {}", e);
            }
        }

        Ok(())
    }

    /// Capture events from a single device (runs in blocking thread)
    fn capture_device(
        mut device: Device,
        sender: broadcast::Sender<InputEvent>,
    ) -> Result<()> {
        let device_name = device.name().unwrap_or("unknown").to_string();
        debug!("Capturing from device: {}", device_name);

        loop {
            // Fetch events (blocking call)
            match device.fetch_events() {
                Ok(events) => {
                    for event in events {
                        if let Some(protocol_event) = convert_event(&event) {
                            // Send to all subscribers
                            let _ = sender.send(protocol_event);
                        }
                    }
                }
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::WouldBlock {
                        // No events available, continue
                        std::thread::sleep(std::time::Duration::from_millis(10));
                        continue;
                    }
                    warn!("Error reading from {}: {}", device_name, e);
                    break;
                }
            }
        }

        Ok(())
    }
}

/// Check if device is a keyboard or mouse
fn is_keyboard_or_mouse(device: &Device) -> bool {
    let supported_events = device.supported_events();

    // Check if device supports KEY events (keyboard or mouse buttons)
    if supported_events.contains(EventType::KEY) {
        // Try to determine if it's a keyboard or mouse by checking for common keys
        if let Some(keys) = device.supported_keys() {
            // Has keyboard keys
            if keys.contains(Key::KEY_A)
                || keys.contains(Key::KEY_ENTER)
                || keys.contains(Key::KEY_SPACE)
            {
                return true;
            }

            // Has mouse buttons
            if keys.contains(Key::BTN_LEFT)
                || keys.contains(Key::BTN_RIGHT)
                || keys.contains(Key::BTN_MIDDLE)
            {
                return true;
            }
        }
    }

    // Check for mouse movement
    if supported_events.contains(EventType::RELATIVE) {
        return true;
    }

    false
}

/// Convert evdev event to protocol event
fn convert_event(event: &evdev::InputEvent) -> Option<InputEvent> {
    let time = event
        .timestamp()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    match event.kind() {
        InputEventKind::Key(key) => {
            // Keyboard or mouse button
            if is_mouse_button(key) {
                Some(InputEvent::MouseButton(MouseButtonEvent {
                    time,
                    button: key.code() as u32,
                    pressed: event.value() == 1,
                }))
            } else {
                Some(InputEvent::Keyboard(KeyboardEvent {
                    time,
                    key: key.code() as u32,
                    pressed: event.value() == 1,
                    modifiers: ModifierState::default(), // TODO: Track modifier state
                }))
            }
        }
        InputEventKind::RelAxis(axis) => {
            // Mouse movement or wheel
            use evdev::RelativeAxisType;

            match axis {
                RelativeAxisType::REL_X | RelativeAxisType::REL_Y => {
                    // Mouse movement - we need to accumulate X and Y
                    // For now, send individual axis movements
                    // TODO: Batch X and Y together
                    let (dx, dy) = if axis == RelativeAxisType::REL_X {
                        (event.value() as f64, 0.0)
                    } else {
                        (0.0, event.value() as f64)
                    };

                    Some(InputEvent::MouseMove(MouseMoveEvent {
                        time,
                        movement: MouseMovement::Relative { dx, dy },
                    }))
                }
                RelativeAxisType::REL_WHEEL | RelativeAxisType::REL_HWHEEL => {
                    let (dx, dy) = if axis == RelativeAxisType::REL_HWHEEL {
                        (event.value() as f64, 0.0)
                    } else {
                        (0.0, event.value() as f64)
                    };

                    Some(InputEvent::MouseWheel(MouseWheelEvent { time, dx, dy }))
                }
                _ => None,
            }
        }
        _ => None,
    }
}

/// Check if key is a mouse button
fn is_mouse_button(key: Key) -> bool {
    matches!(
        key,
        Key::BTN_LEFT
            | Key::BTN_RIGHT
            | Key::BTN_MIDDLE
            | Key::BTN_SIDE
            | Key::BTN_EXTRA
            | Key::BTN_FORWARD
            | Key::BTN_BACK
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_capture_creation() {
        let result = InputCapture::new();
        assert!(result.is_ok());
    }
}
