//! Input event definitions

use serde::{Deserialize, Serialize};

/// Input events that can be shared between machines
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputEvent {
    /// Keyboard key press or release
    Keyboard(KeyboardEvent),
    /// Mouse movement (relative or absolute)
    MouseMove(MouseMoveEvent),
    /// Mouse button press or release
    MouseButton(MouseButtonEvent),
    /// Mouse wheel scroll
    MouseWheel(MouseWheelEvent),
}

/// Keyboard event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyboardEvent {
    /// Timestamp in milliseconds
    pub time: u64,
    /// Key code (Linux evdev code)
    pub key: u32,
    /// True if pressed, false if released
    pub pressed: bool,
    /// Modifier state
    pub modifiers: ModifierState,
}

/// Modifier key state
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct ModifierState {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool,
}

/// Mouse movement event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MouseMoveEvent {
    /// Timestamp in milliseconds
    pub time: u64,
    /// Movement type
    pub movement: MouseMovement,
}

/// Mouse movement types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MouseMovement {
    /// Relative movement (dx, dy)
    Relative { dx: f64, dy: f64 },
    /// Absolute position (x, y) in range [0.0, 1.0]
    Absolute { x: f64, y: f64 },
}

/// Mouse button event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MouseButtonEvent {
    /// Timestamp in milliseconds
    pub time: u64,
    /// Button code (Linux BTN_* codes)
    pub button: u32,
    /// True if pressed, false if released
    pub pressed: bool,
}

/// Mouse wheel event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MouseWheelEvent {
    /// Timestamp in milliseconds
    pub time: u64,
    /// Horizontal scroll amount
    pub dx: f64,
    /// Vertical scroll amount
    pub dy: f64,
}

/// Clipboard data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardData {
    /// MIME type of the data
    pub mime_type: String,
    /// The actual clipboard data
    pub data: Vec<u8>,
}

/// Display information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayInfo {
    /// Display name
    pub name: String,
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
    /// Scale factor
    pub scale: f64,
    /// Position in the virtual screen (x, y)
    pub position: (i32, i32),
}

/// Control messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ControlMessage {
    /// Request to switch focus to this machine
    RequestFocus,
    /// Acknowledge focus switch
    FocusAcquired,
    /// Release focus
    ReleaseFocus,
    /// Disconnect gracefully
    Disconnect,
    /// Error message
    Error(String),
}
