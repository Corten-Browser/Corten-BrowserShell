// @implements: REQ-006
//! Platform-specific implementations
//!
//! This module contains platform-specific implementations for Linux, Windows, and macOS.
//! Each platform module provides a concrete implementation of the PlatformWindow trait.

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "macos")]
pub mod macos;

// Re-export platform-specific window types
#[cfg(target_os = "linux")]
pub use linux::LinuxWindow;

#[cfg(target_os = "windows")]
pub use windows::WindowsWindow;

#[cfg(target_os = "macos")]
pub use macos::MacWindow;
