// @implements: REQ-003, REQ-004, REQ-005
//! Public Browser Shell API
//!
//! Provides a clean, unified API for all browser operations.

use anyhow::{Result, Context};
use std::sync::Arc;
use parking_lot::RwLock;

use shared_types::{WindowId, WindowConfig, TabId, Url, WindowManager, TabManager};
use crate::coordinator::ComponentCoordinator;
use crate::state::BrowserState;

/// BrowserShellAPI provides the public interface for browser operations
#[derive(Clone)]
pub struct BrowserShellAPI {
    coordinator: Arc<ComponentCoordinator>,
    state: Arc<RwLock<BrowserState>>,
}

impl BrowserShellAPI {
    /// Create a new BrowserShellAPI instance
    pub fn new(
        coordinator: Arc<ComponentCoordinator>,
        state: Arc<RwLock<BrowserState>>,
    ) -> Self {
        Self {
            coordinator,
            state,
        }
    }

    /// Create a new window
    pub async fn new_window(&self, config: WindowConfig) -> Result<WindowId> {
        let wm_arc = self.coordinator.window_manager();
        let mut wm = wm_arc.write();
        wm.create_window(config)
            .await
            .context("Failed to create window")
    }

    /// Close a window
    pub async fn close_window(&self, id: WindowId) -> Result<()> {
        let wm_arc = self.coordinator.window_manager();
        let mut wm = wm_arc.write();
        wm.close_window(id)
            .await
            .context("Failed to close window")
    }

    /// Create a new tab in the specified window
    pub async fn new_tab(&self, window_id: WindowId, url: Option<String>) -> Result<TabId> {
        let url = url.map(|s| Url::parse(&s)).transpose()
            .context("Invalid URL")?;

        let tm_arc = self.coordinator.tab_manager();
        let mut tm = tm_arc.write();
        tm.create_tab(window_id, url)
            .await
            .context("Failed to create tab")
    }

    /// Navigate a tab to a URL
    pub async fn navigate(&self, tab_id: TabId, url: String) -> Result<()> {
        let url = Url::parse(&url)
            .context("Invalid URL")?;

        let tm_arc = self.coordinator.tab_manager();
        let mut tm = tm_arc.write();
        tm.navigate(tab_id, url)
            .await
            .context("Failed to navigate")
    }

    /// Reload a tab
    pub async fn reload(&self, tab_id: TabId) -> Result<()> {
        let tm_arc = self.coordinator.tab_manager();
        let mut tm = tm_arc.write();
        tm.reload(tab_id, false)
            .await
            .context("Failed to reload tab")
    }

    /// Go back in navigation history
    pub async fn go_back(&self, tab_id: TabId) -> Result<()> {
        let tm_arc = self.coordinator.tab_manager();
        let mut tm = tm_arc.write();
        tm.go_back(tab_id)
            .await
            .context("Failed to go back")
    }

    /// Go forward in navigation history
    pub async fn go_forward(&self, tab_id: TabId) -> Result<()> {
        let tm_arc = self.coordinator.tab_manager();
        let mut tm = tm_arc.write();
        tm.go_forward(tab_id)
            .await
            .context("Failed to go forward")
    }

    /// Close a tab
    pub async fn close_tab(&self, tab_id: TabId) -> Result<()> {
        let tm_arc = self.coordinator.tab_manager();
        let mut tm = tm_arc.write();
        tm.close_tab(tab_id)
            .await
            .context("Failed to close tab")
    }

    /// Get a setting value
    pub async fn get_setting(&self, key: &str) -> Result<String> {
        let sm_arc = self.coordinator.settings_manager();
        let sm = sm_arc.read();
        sm.get(key)
            .context("Failed to get setting")?
            .ok_or_else(|| anyhow::anyhow!("Setting '{}' not found", key))
    }

    /// Set a setting value
    pub async fn set_setting(&self, key: &str, value: String) -> Result<()> {
        let sm_arc = self.coordinator.settings_manager();
        let mut sm = sm_arc.write();
        sm.set(key, &value)
            .context("Failed to set setting")
    }
}
