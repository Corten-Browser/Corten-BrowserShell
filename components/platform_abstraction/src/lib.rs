//! Platform abstraction layer for browser shell window management
//!
//! This crate provides platform-specific window implementations for Linux, Windows, and macOS.
//! It exposes a common `PlatformWindow` trait that all platform implementations must satisfy,
//! along with platform-specific handle types.
//!
//! # Phase 1: Stub Implementation
//!
//! This is a stub implementation that compiles on all platforms and provides mock functionality.
//! Full native window integration will be implemented in later phases.

mod handles;
mod platform;
mod traits;

// Re-export public types
pub use handles::*;
pub use platform::*;
pub use traits::*;
