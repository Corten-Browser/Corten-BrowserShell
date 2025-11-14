// @implements: REQ-002, REQ-003
//! Tab state tracking
//!
//! Manages the complete state of a tab including navigation, process,
//! and rendering state.

use shared_types::{Tab, TabId, WindowId, ProcessId, RenderSurfaceId, Url};
use crate::navigation::NavigationHistory;

/// Complete state for a single tab
#[derive(Debug, Clone)]
pub struct TabState {
    pub id: TabId,
    pub window_id: WindowId,
    pub title: String,
    pub loading: bool,
    pub history: NavigationHistory,
    pub process_id: Option<ProcessId>,
    pub render_surface: RenderSurfaceId,
    pub favicon: Option<Vec<u8>>,
}

impl TabState {
    /// Create a new tab state
    pub fn new(id: TabId, window_id: WindowId, initial_url: Option<Url>) -> Self {
        let mut history = NavigationHistory::new();

        if let Some(url) = initial_url {
            history.navigate(url);
        }

        Self {
            id,
            window_id,
            title: String::from("New Tab"),
            loading: false,
            history,
            process_id: None,
            render_surface: RenderSurfaceId(id.0 as u64), // Use tab ID for surface
            favicon: None,
        }
    }

    /// Convert to Tab (public API type)
    pub fn to_tab(&self) -> Tab {
        Tab {
            id: self.id,
            window_id: self.window_id,
            title: self.title.clone(),
            url: self.history.current_url().cloned(),
            loading: self.loading,
            can_go_back: self.history.can_go_back(),
            can_go_forward: self.history.can_go_forward(),
            favicon: self.favicon.clone(),
            process_id: self.process_id,
            render_surface: self.render_surface,
        }
    }

    /// Navigate to a URL
    pub fn navigate(&mut self, url: Url) {
        self.history.navigate(url);
        self.loading = true;
    }

    /// Go back in history
    pub fn go_back(&mut self) -> Option<Url> {
        let url = self.history.go_back();
        if url.is_some() {
            self.loading = true;
        }
        url
    }

    /// Go forward in history
    pub fn go_forward(&mut self) -> Option<Url> {
        let url = self.history.go_forward();
        if url.is_some() {
            self.loading = true;
        }
        url
    }

    /// Mark loading as complete
    pub fn finish_loading(&mut self) {
        self.loading = false;
    }

    /// Stop loading
    pub fn stop_loading(&mut self) {
        self.loading = false;
    }

    /// Set process ID
    pub fn set_process(&mut self, process_id: ProcessId) {
        self.process_id = Some(process_id);
    }

    /// Update title
    pub fn set_title(&mut self, title: String) {
        self.title = title;
    }

    /// Set favicon
    pub fn set_favicon(&mut self, favicon: Vec<u8>) {
        self.favicon = Some(favicon);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_tab_state() {
        let tab_id = TabId::new();
        let window_id = WindowId::new();

        let state = TabState::new(tab_id, window_id, None);

        assert_eq!(state.id, tab_id);
        assert_eq!(state.window_id, window_id);
        assert_eq!(state.title, "New Tab");
        assert!(!state.loading);
        assert!(state.process_id.is_none());
    }

    #[test]
    fn test_new_tab_state_with_url() {
        let tab_id = TabId::new();
        let window_id = WindowId::new();
        let url = Url::parse("https://example.com").unwrap();

        let state = TabState::new(tab_id, window_id, Some(url));

        assert_eq!(
            state.history.current_url().map(|u| u.as_str()),
            Some("https://example.com")
        );
    }

    #[test]
    fn test_navigate_updates_history() {
        let tab_id = TabId::new();
        let window_id = WindowId::new();

        let mut state = TabState::new(tab_id, window_id, None);

        let url = Url::parse("https://example.com").unwrap();
        state.navigate(url);

        assert!(state.loading);
        assert_eq!(
            state.history.current_url().map(|u| u.as_str()),
            Some("https://example.com")
        );
    }

    #[test]
    fn test_to_tab_conversion() {
        let tab_id = TabId::new();
        let window_id = WindowId::new();
        let url = Url::parse("https://example.com").unwrap();

        let state = TabState::new(tab_id, window_id, Some(url));
        let tab = state.to_tab();

        assert_eq!(tab.id, tab_id);
        assert_eq!(tab.window_id, window_id);
        assert_eq!(tab.url.as_ref().map(|u| u.as_str()), Some("https://example.com"));
    }

    #[test]
    fn test_set_process() {
        let tab_id = TabId::new();
        let window_id = WindowId::new();

        let mut state = TabState::new(tab_id, window_id, None);

        let process_id = ProcessId(1000);
        state.set_process(process_id);

        assert_eq!(state.process_id, Some(process_id));
    }
}
