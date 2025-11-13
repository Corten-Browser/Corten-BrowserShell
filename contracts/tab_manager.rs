// Tab Manager Contract
// Version: 0.17.0
//
// This contract defines the interface for tab lifecycle and navigation management.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Tab identifier (UUID-based)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TabId(pub u128);

impl TabId {
    pub fn new() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        Self(timestamp)
    }
}

/// Process identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProcessId(pub u32);

/// Render surface identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RenderSurfaceId(pub u64);

/// URL representation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Url(pub String);

impl Url {
    pub fn parse(s: &str) -> Result<Self, TabError> {
        // Basic validation - in real implementation would use url crate
        if s.is_empty() {
            return Err(TabError::InvalidUrl("Empty URL".to_string()));
        }
        Ok(Self(s.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Tab representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tab {
    pub id: TabId,
    pub window_id: super::window_manager::WindowId,
    pub title: String,
    pub url: Option<Url>,
    pub loading: bool,
    pub can_go_back: bool,
    pub can_go_forward: bool,
    pub favicon: Option<Vec<u8>>,
    pub process_id: Option<ProcessId>,
    pub render_surface: RenderSurfaceId,
}

/// Tab error types
#[derive(Debug, thiserror::Error)]
pub enum TabError {
    #[error("Tab not found: {0:?}")]
    NotFound(TabId),

    #[error("Tab creation failed: {0}")]
    CreationFailed(String),

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Navigation failed: {0}")]
    NavigationFailed(String),

    #[error("Window not found: {0:?}")]
    WindowNotFound(super::window_manager::WindowId),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Tab Manager interface
#[async_trait]
pub trait TabManager: Send + Sync {
    /// Create new tab in specified window
    async fn create_tab(
        &mut self,
        window_id: super::window_manager::WindowId,
        url: Option<Url>,
    ) -> Result<TabId, TabError>;

    /// Close tab
    async fn close_tab(&mut self, tab_id: TabId) -> Result<(), TabError>;

    /// Navigate tab to URL
    async fn navigate(&mut self, tab_id: TabId, url: Url) -> Result<(), TabError>;

    /// Reload tab
    async fn reload(&mut self, tab_id: TabId, ignore_cache: bool) -> Result<(), TabError>;

    /// Stop loading
    async fn stop(&mut self, tab_id: TabId) -> Result<(), TabError>;

    /// Navigation history - go back
    async fn go_back(&mut self, tab_id: TabId) -> Result<(), TabError>;

    /// Navigation history - go forward
    async fn go_forward(&mut self, tab_id: TabId) -> Result<(), TabError>;

    /// Get tab information
    fn get_tab(&self, tab_id: TabId) -> Option<&Tab>;

    /// Get all tabs in a window
    fn get_tabs(
        &self,
        window_id: super::window_manager::WindowId,
    ) -> Vec<&Tab>;

    /// Switch active tab
    async fn activate_tab(&mut self, tab_id: TabId) -> Result<(), TabError>;
}
