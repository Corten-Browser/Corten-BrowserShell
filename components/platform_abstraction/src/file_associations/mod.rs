//! File type and protocol association support
//!
//! This module provides a unified API for registering the browser as the default
//! handler for file types and URL protocols across Linux, Windows, and macOS.
//!
//! # Overview
//!
//! The file association system provides:
//! - Registration for file types (.html, .htm, .pdf)
//! - Registration for URL protocols (http://, https://, file://)
//! - Platform-specific implementation (Linux .desktop, Windows Registry, macOS Launch Services)
//! - Capability queries (can register, is registered, is default browser)
//!
//! # Example
//!
//! ```rust,no_run
//! use platform_abstraction::file_associations::{
//!     AssociationService, FileAssociation, SystemAssociationService, AssociationConfig,
//! };
//!
//! // Create a service with custom configuration
//! let config = AssociationConfig::new("My Browser", "my-browser")
//!     .with_executable("/usr/bin/my-browser")
//!     .with_icon("/usr/share/icons/my-browser.png");
//!
//! let service = SystemAssociationService::new(config);
//!
//! // Check capabilities
//! if service.can_register() {
//!     // Register for individual associations
//!     service.register(FileAssociation::HtmlFile).unwrap();
//!     service.register(FileAssociation::HttpsProtocol).unwrap();
//!
//!     // Or register as default browser (HTTP, HTTPS, HTML, HTM)
//!     service.register_as_default_browser().unwrap();
//! }
//!
//! // Query registration status
//! let is_html_handler = service.is_registered(FileAssociation::HtmlFile);
//! let is_default = service.is_default_browser();
//! ```
//!
//! # Platform-Specific Implementation Details
//!
//! ## Linux
//!
//! On Linux, registration uses:
//! - `.desktop` files in `~/.local/share/applications/`
//! - `xdg-mime` for setting default applications
//! - `xdg-settings` for default browser detection
//!
//! ## Windows
//!
//! On Windows, registration uses:
//! - Windows Registry modifications
//! - User-level or system-level registration depending on permissions
//!
//! ## macOS
//!
//! On macOS, registration uses:
//! - Launch Services framework
//! - `Info.plist` for app bundle configuration
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
    associations_supported, AssociationError, AssociationResult, AssociationService,
    SystemAssociationService,
};
pub use types::{AssociationConfig, AssociationStatus, FileAssociation};
