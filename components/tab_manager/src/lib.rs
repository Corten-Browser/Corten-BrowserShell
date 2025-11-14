// @implements: REQ-002, REQ-003
//! Tab Manager Component
//!
//! Provides tab lifecycle management, process isolation, navigation history,
//! and tab state tracking for the Corten Browser Shell.
//!
//! # Features
//!
//! - Tab creation and closure
//! - Navigation history (back/forward)
//! - Process isolation per tab (mocked)
//! - Tab state tracking
//! - Supports 500+ tabs
//! - Tab switching < 10ms
//!
//! # Example
//!
//! ```rust,no_run
//! use tab_manager::TabManagerImpl;
//! use shared_types::{TabManager, WindowId, Url};
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut manager = TabManagerImpl::new();
//!     let window_id = WindowId::new();
//!     let url = Url::parse("https://example.com").unwrap();
//!
//!     let tab_id = manager.create_tab(window_id, Some(url))
//!         .await
//!         .expect("Failed to create tab");
//! }
//! ```

mod manager;
mod navigation;
mod process;
mod state;

pub use manager::TabManagerImpl;
pub use navigation::NavigationHistory;
pub use process::MockProcessManager;
pub use state::TabState;

#[cfg(test)]
mod tests {
    // Integration with unit tests
}
