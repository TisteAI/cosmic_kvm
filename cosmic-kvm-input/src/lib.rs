//! Input abstraction layer for COSMIC KVM
//!
//! Provides a unified interface for virtual input injection with multiple backends:
//! - uinput: Universal fallback using Linux uinput interface
//! - Wayland: Native Wayland virtual input protocols (future implementation)

use cosmic_kvm_protocol::{InputEvent, KeyboardEvent, MouseButtonEvent, MouseMoveEvent, MouseWheelEvent};
use thiserror::Error;

pub mod backends;

#[cfg(feature = "uinput")]
pub use backends::uinput::UInputBackend;

#[cfg(feature = "wayland")]
pub use backends::wayland::WaylandBackend;

/// Result type for input operations
pub type Result<T> = std::result::Result<T, InputError>;

/// Errors that can occur during input operations
#[derive(Debug, Error)]
pub enum InputError {
    #[error("Backend initialization failed: {0}")]
    InitializationFailed(String),

    #[error("Device creation failed: {0}")]
    DeviceCreationFailed(String),

    #[error("Event injection failed: {0}")]
    EventInjectionFailed(String),

    #[error("Unsupported event type: {0}")]
    UnsupportedEventType(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[cfg(feature = "uinput")]
    #[error("evdev error: {0}")]
    Evdev(#[from] evdev::Error),
}

/// Trait for input injection backends
pub trait InputBackend: Send + Sync {
    /// Initialize the backend
    fn init(&mut self) -> Result<()>;

    /// Inject a keyboard event
    fn inject_keyboard(&mut self, event: &KeyboardEvent) -> Result<()>;

    /// Inject a mouse movement event
    fn inject_mouse_move(&mut self, event: &MouseMoveEvent) -> Result<()>;

    /// Inject a mouse button event
    fn inject_mouse_button(&mut self, event: &MouseButtonEvent) -> Result<()>;

    /// Inject a mouse wheel event
    fn inject_mouse_wheel(&mut self, event: &MouseWheelEvent) -> Result<()>;

    /// Inject a generic input event
    fn inject_event(&mut self, event: &InputEvent) -> Result<()> {
        match event {
            InputEvent::Keyboard(e) => self.inject_keyboard(e),
            InputEvent::MouseMove(e) => self.inject_mouse_move(e),
            InputEvent::MouseButton(e) => self.inject_mouse_button(e),
            InputEvent::MouseWheel(e) => self.inject_mouse_wheel(e),
        }
    }

    /// Clean up resources
    fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}

/// Auto-detecting backend selector
pub struct InputManager {
    backend: Box<dyn InputBackend>,
}

impl InputManager {
    /// Create a new input manager with the best available backend
    pub fn new() -> Result<Self> {
        Self::with_preferred_backend(None)
    }

    /// Create a new input manager with a preferred backend
    pub fn with_preferred_backend(preferred: Option<BackendType>) -> Result<Self> {
        let backend: Box<dyn InputBackend> = match preferred {
            #[cfg(feature = "wayland")]
            Some(BackendType::Wayland) => {
                tracing::info!("Using Wayland backend for input injection");
                Box::new(WaylandBackend::new()?)
            }
            #[cfg(feature = "uinput")]
            Some(BackendType::UInput) | None => {
                tracing::info!("Using uinput backend for input injection");
                Box::new(UInputBackend::new()?)
            }
            #[cfg(not(feature = "uinput"))]
            None => {
                return Err(InputError::InitializationFailed(
                    "No input backend available".to_string(),
                ))
            }
        };

        let mut manager = Self { backend };
        manager.backend.init()?;
        Ok(manager)
    }

    /// Inject an input event
    pub fn inject(&mut self, event: &InputEvent) -> Result<()> {
        self.backend.inject_event(event)
    }
}

impl Drop for InputManager {
    fn drop(&mut self) {
        let _ = self.backend.shutdown();
    }
}

/// Available backend types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendType {
    #[cfg(feature = "wayland")]
    Wayland,
    #[cfg(feature = "uinput")]
    UInput,
}
