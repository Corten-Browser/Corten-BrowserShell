// @implements: REQ-002, REQ-003
//! Tab management types
//!
//! This module provides types for tab lifecycle and navigation management.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::error::TabError;
use crate::window::WindowId;

/// Tab identifier (UUID-based)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TabId(pub u128);

impl TabId {
    /// Create a new unique TabId
    pub fn new() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        Self(timestamp)
    }
}

impl Default for TabId {
    fn default() -> Self {
        Self::new()
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
    /// Parse a string into a URL
    pub fn parse(s: &str) -> Result<Self, TabError> {
        // Basic validation - in real implementation would use url crate
        if s.is_empty() {
            return Err(TabError::InvalidUrl("Empty URL".to_string()));
        }
        Ok(Self(s.to_string()))
    }

    /// Get the URL as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Tab representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tab {
    pub id: TabId,
    pub window_id: WindowId,
    pub title: String,
    pub url: Option<Url>,
    pub loading: bool,
    pub can_go_back: bool,
    pub can_go_forward: bool,
    pub favicon: Option<Vec<u8>>,
    pub process_id: Option<ProcessId>,
    pub render_surface: RenderSurfaceId,
}

/// Tab Manager interface
#[async_trait]
pub trait TabManager: Send + Sync {
    /// Create new tab in specified window
    async fn create_tab(
        &mut self,
        window_id: WindowId,
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
    fn get_tabs(&self, window_id: WindowId) -> Vec<&Tab>;

    /// Switch active tab
    async fn activate_tab(&mut self, tab_id: TabId) -> Result<(), TabError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tab_id_creation_is_unique() {
        let id1 = TabId::new();
        std::thread::sleep(std::time::Duration::from_nanos(10));
        let id2 = TabId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn tab_id_default_creates_new() {
        let id = TabId::default();
        assert!(id.0 > 0);
    }

    #[test]
    fn url_parse_rejects_empty() {
        let result = Url::parse("");
        assert!(result.is_err());
    }

    #[test]
    fn url_parse_accepts_valid() {
        let result = Url::parse("https://example.com");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_str(), "https://example.com");
    }

    #[test]
    fn url_as_str_returns_reference() {
        let url = Url("https://test.com".to_string());
        assert_eq!(url.as_str(), "https://test.com");
    }
}
