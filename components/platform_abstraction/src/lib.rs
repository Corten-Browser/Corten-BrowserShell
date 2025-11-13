// @implements: REQ-001, REQ-002, REQ-003, REQ-004, REQ-005, REQ-006
//! Platform Abstraction Library
//!
//! Cross-platform window operations, clipboard, notifications, and system integration
//! for Linux, Windows, and macOS.
//!
//! # Modules
//!
//! - `platform_window`: Platform window creation trait and implementations
//! - `events`: Platform event translation
//! - `clipboard`: Clipboard operations
//! - `notifications`: System notifications
//! - `platform`: Platform-specific implementations (Linux, Windows, macOS)
//!
//! # Example
//!
//! ```rust,no_run
//! use platform_abstraction::platform_window::{MockPlatformWindow, PlatformWindow};
//! use shared_types::window::WindowConfig;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let mut window = MockPlatformWindow::new();
//! let config = WindowConfig::default();
//! let window_id = window.create(config).await?;
//! # Ok(())
//! # }
//! ```

// Module declarations (will be implemented in GREEN phase)
pub mod platform_window;
pub mod events;
pub mod clipboard;
pub mod notifications;
pub mod platform;

// Re-export commonly used types
pub use platform_window::{PlatformWindow, MockPlatformWindow};
pub use events::EventTranslator;
pub use clipboard::Clipboard;
pub use notifications::{Notifier, Notification};
