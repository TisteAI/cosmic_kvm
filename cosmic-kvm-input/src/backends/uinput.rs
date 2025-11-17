//! uinput backend for virtual input device creation
//!
//! This backend uses Linux's uinput interface to create virtual input devices.
//! It works universally across X11, Wayland, and even console, but requires
//! appropriate permissions (typically group membership or root).

use crate::{InputBackend, InputError, Result};
use cosmic_kvm_protocol::{
    KeyboardEvent, MouseButtonEvent, MouseMoveEvent, MouseMovement, MouseWheelEvent,
};
use evdev::{
    uinput::{VirtualDevice, VirtualDeviceBuilder},
    AbsInfo, AbsoluteAxisType, AttributeSet, EventType, InputEvent as EvdevInputEvent, Key,
    RelativeAxisType, UinputAbsSetup,
};

/// uinput backend implementation
pub struct UInputBackend {
    keyboard: Option<VirtualDevice>,
    mouse: Option<VirtualDevice>,
    screen_width: i32,
    screen_height: i32,
}

impl UInputBackend {
    /// Create a new uinput backend
    pub fn new() -> Result<Self> {
        Ok(Self {
            keyboard: None,
            mouse: None,
            screen_width: 1920, // Default, should be queried from display info
            screen_height: 1080,
        })
    }

    /// Set the screen dimensions for absolute positioning
    pub fn set_screen_size(&mut self, width: i32, height: i32) {
        self.screen_width = width;
        self.screen_height = height;
    }

    /// Create virtual keyboard device
    fn create_keyboard(&self) -> Result<VirtualDevice> {
        // Add all keyboard keys
        let mut keys = AttributeSet::<Key>::new();

        // Add all standard keys (KEY_ESC through KEY_MICMUTE)
        for code in 1..=256 {
            let key = Key::new(code);
            keys.insert(key);
        }

        let device = VirtualDeviceBuilder::new()
            .map_err(|e| InputError::DeviceCreationFailed(e.to_string()))?
            .name("COSMIC KVM Virtual Keyboard")
            .with_keys(&keys)
            .map_err(|e| InputError::DeviceCreationFailed(e.to_string()))?
            .build()
            .map_err(|e| InputError::DeviceCreationFailed(e.to_string()))?;

        tracing::info!("Created virtual keyboard device");
        Ok(device)
    }

    /// Create virtual mouse device with both relative and absolute positioning
    fn create_mouse(&self) -> Result<VirtualDevice> {
        // Mouse buttons
        let mut keys = AttributeSet::<Key>::new();
        keys.insert(Key::BTN_LEFT);
        keys.insert(Key::BTN_RIGHT);
        keys.insert(Key::BTN_MIDDLE);
        keys.insert(Key::BTN_SIDE);
        keys.insert(Key::BTN_EXTRA);

        // Relative axes for movement
        let mut relative = AttributeSet::<RelativeAxisType>::new();
        relative.insert(RelativeAxisType::REL_X);
        relative.insert(RelativeAxisType::REL_Y);
        relative.insert(RelativeAxisType::REL_WHEEL);
        relative.insert(RelativeAxisType::REL_HWHEEL);

        // Absolute axes for absolute positioning
        let abs_x = UinputAbsSetup::new(
            AbsoluteAxisType::ABS_X,
            AbsInfo::new(0, 0, self.screen_width, 0, 0, 1),
        );

        let abs_y = UinputAbsSetup::new(
            AbsoluteAxisType::ABS_Y,
            AbsInfo::new(0, 0, self.screen_height, 0, 0, 1),
        );

        let device = VirtualDeviceBuilder::new()
            .map_err(|e| InputError::DeviceCreationFailed(e.to_string()))?
            .name("COSMIC KVM Virtual Mouse")
            .with_keys(&keys)
            .map_err(|e| InputError::DeviceCreationFailed(e.to_string()))?
            .with_relative_axes(&relative)
            .map_err(|e| InputError::DeviceCreationFailed(e.to_string()))?
            .with_absolute_axis(&abs_x)
            .map_err(|e| InputError::DeviceCreationFailed(e.to_string()))?
            .with_absolute_axis(&abs_y)
            .map_err(|e| InputError::DeviceCreationFailed(e.to_string()))?
            .build()
            .map_err(|e| InputError::DeviceCreationFailed(e.to_string()))?;

        tracing::info!("Created virtual mouse device");
        Ok(device)
    }

}

impl InputBackend for UInputBackend {
    fn init(&mut self) -> Result<()> {
        // Check uinput access
        if !std::path::Path::new("/dev/uinput").exists() {
            return Err(InputError::InitializationFailed(
                "uinput device not found. Is the uinput module loaded?".to_string(),
            ));
        }

        // Create virtual devices
        self.keyboard = Some(self.create_keyboard()?);
        self.mouse = Some(self.create_mouse()?);

        tracing::info!("uinput backend initialized successfully");
        Ok(())
    }

    fn inject_keyboard(&mut self, event: &KeyboardEvent) -> Result<()> {
        let keyboard = self
            .keyboard
            .as_mut()
            .ok_or_else(|| InputError::EventInjectionFailed("Keyboard not initialized".into()))?;

        let key = Key::new(event.key as u16);
        let value = if event.pressed { 1 } else { 0 };

        let events = [
            EvdevInputEvent::new_now(EventType::KEY, key.code(), value),
            EvdevInputEvent::new_now(EventType::SYNCHRONIZATION, 0, 0),
        ];

        keyboard
            .emit(&events)
            .map_err(|e| InputError::EventInjectionFailed(e.to_string()))?;

        Ok(())
    }

    fn inject_mouse_move(&mut self, event: &MouseMoveEvent) -> Result<()> {
        let mouse = self
            .mouse
            .as_mut()
            .ok_or_else(|| InputError::EventInjectionFailed("Mouse not initialized".into()))?;

        let events = match &event.movement {
            MouseMovement::Relative { dx, dy } => {
                vec![
                    EvdevInputEvent::new_now(
                        EventType::RELATIVE,
                        RelativeAxisType::REL_X.0,
                        *dx as i32,
                    ),
                    EvdevInputEvent::new_now(
                        EventType::RELATIVE,
                        RelativeAxisType::REL_Y.0,
                        *dy as i32,
                    ),
                    EvdevInputEvent::new_now(EventType::SYNCHRONIZATION, 0, 0),
                ]
            }
            MouseMovement::Absolute { x, y } => {
                // Convert normalized coordinates to screen coordinates
                let abs_x = (*x * self.screen_width as f64) as i32;
                let abs_y = (*y * self.screen_height as f64) as i32;

                vec![
                    EvdevInputEvent::new_now(
                        EventType::ABSOLUTE,
                        AbsoluteAxisType::ABS_X.0,
                        abs_x,
                    ),
                    EvdevInputEvent::new_now(
                        EventType::ABSOLUTE,
                        AbsoluteAxisType::ABS_Y.0,
                        abs_y,
                    ),
                    EvdevInputEvent::new_now(EventType::SYNCHRONIZATION, 0, 0),
                ]
            }
        };

        mouse
            .emit(&events)
            .map_err(|e| InputError::EventInjectionFailed(e.to_string()))?;

        Ok(())
    }

    fn inject_mouse_button(&mut self, event: &MouseButtonEvent) -> Result<()> {
        let mouse = self
            .mouse
            .as_mut()
            .ok_or_else(|| InputError::EventInjectionFailed("Mouse not initialized".into()))?;

        let button = Key::new(event.button as u16);
        let value = if event.pressed { 1 } else { 0 };

        let events = [
            EvdevInputEvent::new_now(EventType::KEY, button.code(), value),
            EvdevInputEvent::new_now(EventType::SYNCHRONIZATION, 0, 0),
        ];

        mouse
            .emit(&events)
            .map_err(|e| InputError::EventInjectionFailed(e.to_string()))?;

        Ok(())
    }

    fn inject_mouse_wheel(&mut self, event: &MouseWheelEvent) -> Result<()> {
        let mouse = self
            .mouse
            .as_mut()
            .ok_or_else(|| InputError::EventInjectionFailed("Mouse not initialized".into()))?;

        let mut events = Vec::new();

        if event.dy != 0.0 {
            events.push(EvdevInputEvent::new_now(
                EventType::RELATIVE,
                RelativeAxisType::REL_WHEEL.0,
                event.dy as i32,
            ));
        }

        if event.dx != 0.0 {
            events.push(EvdevInputEvent::new_now(
                EventType::RELATIVE,
                RelativeAxisType::REL_HWHEEL.0,
                event.dx as i32,
            ));
        }

        events.push(EvdevInputEvent::new_now(EventType::SYNCHRONIZATION, 0, 0));

        mouse
            .emit(&events)
            .map_err(|e| InputError::EventInjectionFailed(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_creation() {
        let backend = UInputBackend::new();
        assert!(backend.is_ok());
    }
}
