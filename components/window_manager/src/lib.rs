// @implements: REQ-001, REQ-002, REQ-003, REQ-004, REQ-005, REQ-006
//! Window Manager Component
//!
//! Provides window lifecycle management, multi-window support, and window state coordination.
//!
//! # Features
//!
//! - Window creation and destruction
//! - Multi-window support (50+ concurrent windows)
//! - Window state management (size, position, fullscreen)
//! - Platform event handling
//! - Window state persistence
//!
//! # Example
//!
//! ```rust
//! use window_manager::WindowManagerImpl;
//! use shared_types::window::{WindowManager, WindowConfig};
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut manager = WindowManagerImpl::new();
//!     let config = WindowConfig::default();
//!     let window_id = manager.create_window(config).await.unwrap();
//!     println!("Created window: {:?}", window_id);
//! }
//! ```

mod manager;
mod window_state;
mod events;

pub use manager::WindowManagerImpl;
pub use window_state::WindowState;
pub use events::EventHandler;