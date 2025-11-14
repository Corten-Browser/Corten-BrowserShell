// @implements: REQ-001, REQ-002, REQ-003, REQ-004, REQ-005, REQ-006
//! Browser Shell - Main browser orchestration and component coordination
//!
//! This is the top-level integration component that ties all browser components together.
//! It provides a unified public API for window and tab operations, manages component lifecycle,
//! and coordinates message routing between components.
//!
//! # Architecture
//!
//! The BrowserShell acts as the central coordinator:
//! - Initializes all components (message_bus, platform_abstraction, window_manager, etc.)
//! - Manages component lifecycle (startup, shutdown)
//! - Routes messages between components via the message bus
//! - Exposes a clean public API (BrowserShellAPI)
//! - Monitors component health
//!
//! # Example
//!
//! ```rust
//! use browser_shell::BrowserShell;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Create and initialize the browser
//!     let browser = BrowserShell::new().await?;
//!
//!     // Use the public API
//!     let api = browser.api();
//!     let window_id = api.new_window(Default::default()).await?;
//!     let tab_id = api.new_tab(window_id, Some("https://example.com".to_string())).await?;
//!
//!     // Shutdown cleanly
//!     browser.shutdown().await?;
//!     Ok(())
//! }
//! ```

use std::sync::Arc;
use parking_lot::RwLock;
use anyhow::Result;

mod coordinator;
mod api;
mod state;

pub use api::BrowserShellAPI;
pub use coordinator::ComponentCoordinator;
pub use state::BrowserState;

// Re-export commonly used types from shared_types
pub use shared_types::{
    WindowId, WindowConfig, TabId,
    ComponentHealth, ComponentError,
};

/// Main BrowserShell struct
///
/// This is the top-level orchestrator that manages all browser components.
/// It provides lifecycle management and exposes the public API.
pub struct BrowserShell {
    coordinator: Arc<ComponentCoordinator>,
    state: Arc<RwLock<BrowserState>>,
    api: BrowserShellAPI,
}

impl BrowserShell {
    /// Create and initialize a new BrowserShell instance
    ///
    /// This initializes all components in the correct order:
    /// 1. Shared types and message bus
    /// 2. Platform abstraction
    /// 3. Window and tab managers
    /// 4. UI chrome components
    /// 5. User data management
    ///
    /// # Errors
    ///
    /// Returns an error if any component fails to initialize.
    pub async fn new() -> Result<Self> {
        // Create the component coordinator
        let coordinator = Arc::new(ComponentCoordinator::new().await?);

        // Create the browser state
        let state = Arc::new(RwLock::new(BrowserState::new()));

        // Create the public API
        let api = BrowserShellAPI::new(
            Arc::clone(&coordinator),
            Arc::clone(&state),
        );

        Ok(Self {
            coordinator,
            state,
            api,
        })
    }

    /// Get a reference to the public API
    ///
    /// The API provides methods for window and tab operations,
    /// settings management, and other browser functionality.
    pub fn api(&self) -> &BrowserShellAPI {
        &self.api
    }

    /// Check the health of all components
    ///
    /// Returns the aggregated health status of all browser components.
    ///
    /// # Errors
    ///
    /// Returns an error if any component reports an unhealthy status.
    pub async fn health_check(&self) -> Result<ComponentHealth> {
        self.coordinator.health_check().await
    }

    /// Shutdown the browser cleanly
    ///
    /// This shuts down all components in reverse initialization order,
    /// ensuring clean cleanup of resources.
    ///
    /// # Errors
    ///
    /// Returns an error if any component fails to shutdown cleanly.
    pub async fn shutdown(self) -> Result<()> {
        self.coordinator.shutdown().await
    }
}
