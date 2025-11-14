//! Platform-specific window implementations
//!
//! This module contains stub implementations for each supported platform.
//! These are minimal implementations that satisfy the PlatformWindow trait
//! interface while logging operations for testing purposes.

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "macos")]
mod macos;

// Re-export platform-specific window types
#[cfg(target_os = "linux")]
pub use linux::LinuxWindow;

#[cfg(target_os = "windows")]
pub use windows::WindowsWindow;

#[cfg(target_os = "macos")]
pub use macos::MacWindow;
