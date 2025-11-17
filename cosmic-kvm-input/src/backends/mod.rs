//! Input backend implementations

#[cfg(feature = "uinput")]
pub mod uinput;

#[cfg(feature = "wayland")]
pub mod wayland;
