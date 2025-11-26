//! URL protocol handler support
//!
//! This module provides a unified API for registering the browser as the handler
//! for URL protocol schemes across Linux, Windows, and macOS.
//!
//! # Overview
//!
//! The protocol handler system provides:
//! - Registration for URL protocols (http://, https://, file://, custom://)
//! - Platform-specific implementation (Linux .desktop, Windows Registry, macOS Launch Services)
//! - URL handling and invocation
//! - Capability queries (can register, is registered, is default browser)
//!
//! # Example
//!
//! ```rust,no_run
//! use platform_abstraction::protocol_handlers::{
//!     ProtocolHandler, Protocol, SystemProtocolHandler, ProtocolConfig,
//! };
//!
//! // Create a handler with custom configuration
//! let config = ProtocolConfig::new("My Browser", "my-browser")
//!     .with_executable("/usr/bin/my-browser")
//!     .with_icon("/usr/share/icons/my-browser.png")
//!     .with_command_template("%u");
//!
//! let handler = SystemProtocolHandler::with_config(config);
//!
//! // Check capabilities
//! if handler.can_register() {
//!     // Register for individual protocols
//!     handler.register_protocol("http").unwrap();
//!     handler.register_protocol("https").unwrap();
//!
//!     // Or register for all web protocols at once
//!     handler.register_web_protocols().unwrap();
//! }
//!
//! // Query registration status
//! let is_https_handler = handler.is_registered("https");
//! let is_web_browser = handler.is_registered_web_browser();
//!
//! // Handle a URL
//! handler.handle_url("https://example.com/page").unwrap();
//! ```
//!
//! # Custom Protocol Registration
//!
//! ```rust,no_run
//! use platform_abstraction::protocol_handlers::{
//!     ProtocolHandler, SystemProtocolHandler,
//! };
//!
//! let handler = SystemProtocolHandler::new();
//!
//! // Register a custom protocol scheme
//! handler.register_protocol("myapp").unwrap();
//!
//! // Check if registered
//! if handler.is_registered("myapp") {
//!     println!("Custom protocol registered");
//! }
//!
//! // Handle custom protocol URL
//! handler.handle_url("myapp://action/data").unwrap();
//! ```
//!
//! # Platform-Specific Implementation Details
//!
//! ## Linux
//!
//! On Linux, registration uses:
//! - `.desktop` files in `~/.local/share/applications/`
//! - `xdg-mime` for MIME type associations
//! - `xdg-settings` for default browser detection
//! - Protocol handlers specified in desktop file with `x-scheme-handler/<protocol>`
//!
//! ## Windows
//!
//! On Windows, registration uses:
//! - Windows Registry modifications
//! - Protocol registration under `HKEY_CLASSES_ROOT\<protocol>`
//! - User-level or system-level registration depending on permissions
//!
//! ## macOS
//!
//! On macOS, registration uses:
//! - Launch Services framework
//! - `Info.plist` configuration in app bundle
//! - `CFBundleURLTypes` for protocol declarations
//!
//! # Current Implementation Status
//!
//! This is currently a stub implementation that:
//! - Compiles on all platforms
//! - Provides the complete API surface
//! - Returns success for registration operations (without actual system changes)
//! - Returns `false` for status queries
//! - Logs intended operations for debugging
//!
//! Full platform integration will be implemented in later phases.

mod service;
mod types;

// Re-export public types
pub use service::{
    protocol_handling_supported, ProtocolError, ProtocolHandler, ProtocolResult,
    SystemProtocolHandler,
};
pub use types::{Protocol, ProtocolConfig, ProtocolStatus, ProtocolUrl};
