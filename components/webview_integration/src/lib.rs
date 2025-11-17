//! WebView Integration Component
//!
//! Provides web content rendering coordination using wry WebView.
//! Manages page loading, JavaScript bridge, resource caching,
//! and navigation events.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use url::Url;

#[derive(Error, Debug)]
pub enum WebViewError {
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    #[error("Navigation failed: {0}")]
    NavigationFailed(String),
    #[error("JavaScript execution failed: {0}")]
    JsError(String),
    #[error("WebView not initialized")]
    NotInitialized,
    #[error("Resource not found: {0}")]
    ResourceNotFound(String),
}

pub type Result<T> = std::result::Result<T, WebViewError>;

/// Navigation event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NavigationEvent {
    Started { url: String, timestamp: DateTime<Utc> },
    Committed { url: String, timestamp: DateTime<Utc> },
    Completed { url: String, timestamp: DateTime<Utc> },
    Failed { url: String, error: String, timestamp: DateTime<Utc> },
}

/// Page load state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoadState {
    Idle,
    Loading,
    Interactive,
    Complete,
    Failed,
}

/// WebView instance state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebViewState {
    pub id: u64,
    pub current_url: String,
    pub title: String,
    pub load_state: LoadState,
    pub can_go_back: bool,
    pub can_go_forward: bool,
    pub is_loading: bool,
    pub zoom_level: f32,
}

impl Default for WebViewState {
    fn default() -> Self {
        Self {
            id: 0,
            current_url: "about:blank".to_string(),
            title: "New Tab".to_string(),
            load_state: LoadState::Idle,
            can_go_back: false,
            can_go_forward: false,
            is_loading: false,
            zoom_level: 1.0,
        }
    }
}

/// Cached resource
#[derive(Debug, Clone)]
struct CachedResource {
    data: Vec<u8>,
    mime_type: String,
    cached_at: DateTime<Utc>,
    expires_at: Option<DateTime<Utc>>,
}

/// JavaScript message from web page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsMessage {
    pub id: String,
    pub method: String,
    pub params: serde_json::Value,
}

/// JavaScript response to web page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsResponse {
    pub id: String,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
}

/// WebView configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebViewConfig {
    /// Enable JavaScript
    pub javascript_enabled: bool,
    /// Enable developer tools
    pub devtools_enabled: bool,
    /// User agent string
    pub user_agent: String,
    /// Enable clipboard access
    pub clipboard_enabled: bool,
    /// Enable autoplay
    pub autoplay_enabled: bool,
    /// Cache size in bytes
    pub cache_size: usize,
}

impl Default for WebViewConfig {
    fn default() -> Self {
        Self {
            javascript_enabled: true,
            devtools_enabled: false,
            user_agent: "CortenBrowser/0.4.0".to_string(),
            clipboard_enabled: true,
            autoplay_enabled: false,
            cache_size: 100 * 1024 * 1024, // 100MB
        }
    }
}

/// WebView Manager for coordinating web content rendering
pub struct WebViewManager {
    /// Active WebView states (id -> state)
    views: Arc<RwLock<HashMap<u64, WebViewState>>>,
    /// Navigation history stack per view (id -> history)
    history: Arc<RwLock<HashMap<u64, Vec<String>>>>,
    /// Current history position per view (id -> position)
    history_position: Arc<RwLock<HashMap<u64, usize>>>,
    /// Resource cache
    cache: Arc<RwLock<HashMap<String, CachedResource>>>,
    /// Configuration
    config: Arc<RwLock<WebViewConfig>>,
    /// Event listeners
    navigation_events: Arc<RwLock<Vec<NavigationEvent>>>,
    /// Next view ID
    next_id: Arc<RwLock<u64>>,
}

impl WebViewManager {
    /// Create a new WebViewManager
    pub fn new() -> Self {
        Self {
            views: Arc::new(RwLock::new(HashMap::new())),
            history: Arc::new(RwLock::new(HashMap::new())),
            history_position: Arc::new(RwLock::new(HashMap::new())),
            cache: Arc::new(RwLock::new(HashMap::new())),
            config: Arc::new(RwLock::new(WebViewConfig::default())),
            navigation_events: Arc::new(RwLock::new(Vec::new())),
            next_id: Arc::new(RwLock::new(1)),
        }
    }

    /// Create a new WebView instance
    pub async fn create_webview(&self) -> u64 {
        let mut next_id = self.next_id.write().await;
        let id = *next_id;
        *next_id += 1;

        let state = WebViewState {
            id,
            ..Default::default()
        };

        let mut views = self.views.write().await;
        views.insert(id, state);

        let mut history = self.history.write().await;
        history.insert(id, vec!["about:blank".to_string()]);

        let mut positions = self.history_position.write().await;
        positions.insert(id, 0);

        id
    }

    /// Destroy a WebView instance
    pub async fn destroy_webview(&self, id: u64) -> Result<()> {
        let mut views = self.views.write().await;
        if views.remove(&id).is_none() {
            return Err(WebViewError::NotInitialized);
        }

        let mut history = self.history.write().await;
        history.remove(&id);

        let mut positions = self.history_position.write().await;
        positions.remove(&id);

        Ok(())
    }

    /// Navigate to a URL
    pub async fn navigate(&self, id: u64, url: String) -> Result<()> {
        // Validate URL
        let parsed_url = Url::parse(&url)
            .map_err(|e| WebViewError::InvalidUrl(e.to_string()))?;

        let mut views = self.views.write().await;
        let view = views
            .get_mut(&id)
            .ok_or(WebViewError::NotInitialized)?;

        // Record navigation start event
        let start_event = NavigationEvent::Started {
            url: url.clone(),
            timestamp: Utc::now(),
        };
        let mut events = self.navigation_events.write().await;
        events.push(start_event);
        drop(events);

        // Update state
        view.current_url = url.clone();
        view.load_state = LoadState::Loading;
        view.is_loading = true;
        view.title = parsed_url.host_str().unwrap_or("Loading...").to_string();

        // Update history
        drop(views);
        let mut history = self.history.write().await;
        let mut positions = self.history_position.write().await;

        if let Some(hist) = history.get_mut(&id) {
            if let Some(pos) = positions.get_mut(&id) {
                // Truncate forward history
                hist.truncate(*pos + 1);
                // Add new URL
                hist.push(url.clone());
                *pos = hist.len() - 1;
            }
        }
        drop(history);
        drop(positions);

        // Update navigation state
        self.update_navigation_state(id).await;

        // Simulate load completion (in real impl, this would be triggered by actual page load)
        let mut views = self.views.write().await;
        if let Some(view) = views.get_mut(&id) {
            view.load_state = LoadState::Complete;
            view.is_loading = false;
        }

        // Record completion event
        let complete_event = NavigationEvent::Completed {
            url,
            timestamp: Utc::now(),
        };
        let mut events = self.navigation_events.write().await;
        events.push(complete_event);

        Ok(())
    }

    /// Navigate back in history
    pub async fn go_back(&self, id: u64) -> Result<()> {
        let can_go_back = {
            let views = self.views.read().await;
            let view = views.get(&id).ok_or(WebViewError::NotInitialized)?;
            view.can_go_back
        };

        if !can_go_back {
            return Err(WebViewError::NavigationFailed("Cannot go back".to_string()));
        }

        let url = {
            let mut positions = self.history_position.write().await;
            let history = self.history.read().await;

            let pos = positions.get_mut(&id).ok_or(WebViewError::NotInitialized)?;
            let hist = history.get(&id).ok_or(WebViewError::NotInitialized)?;

            if *pos > 0 {
                *pos -= 1;
                hist.get(*pos).cloned().ok_or(WebViewError::NavigationFailed(
                    "History corrupted".to_string(),
                ))?
            } else {
                return Err(WebViewError::NavigationFailed("Cannot go back".to_string()));
            }
        };

        // Navigate to the URL without adding to history
        self.navigate_without_history(id, url).await?;
        self.update_navigation_state(id).await;

        Ok(())
    }

    /// Navigate forward in history
    pub async fn go_forward(&self, id: u64) -> Result<()> {
        let can_go_forward = {
            let views = self.views.read().await;
            let view = views.get(&id).ok_or(WebViewError::NotInitialized)?;
            view.can_go_forward
        };

        if !can_go_forward {
            return Err(WebViewError::NavigationFailed("Cannot go forward".to_string()));
        }

        let url = {
            let mut positions = self.history_position.write().await;
            let history = self.history.read().await;

            let pos = positions.get_mut(&id).ok_or(WebViewError::NotInitialized)?;
            let hist = history.get(&id).ok_or(WebViewError::NotInitialized)?;

            if *pos < hist.len() - 1 {
                *pos += 1;
                hist.get(*pos).cloned().ok_or(WebViewError::NavigationFailed(
                    "History corrupted".to_string(),
                ))?
            } else {
                return Err(WebViewError::NavigationFailed("Cannot go forward".to_string()));
            }
        };

        self.navigate_without_history(id, url).await?;
        self.update_navigation_state(id).await;

        Ok(())
    }

    /// Navigate without adding to history (for back/forward)
    async fn navigate_without_history(&self, id: u64, url: String) -> Result<()> {
        let mut views = self.views.write().await;
        let view = views.get_mut(&id).ok_or(WebViewError::NotInitialized)?;

        view.current_url = url;
        view.load_state = LoadState::Complete;
        view.is_loading = false;

        Ok(())
    }

    /// Update can_go_back and can_go_forward state
    async fn update_navigation_state(&self, id: u64) {
        let history = self.history.read().await;
        let positions = self.history_position.read().await;

        if let (Some(hist), Some(pos)) = (history.get(&id), positions.get(&id)) {
            let can_back = *pos > 0;
            let can_forward = *pos < hist.len() - 1;

            drop(history);
            drop(positions);

            let mut views = self.views.write().await;
            if let Some(view) = views.get_mut(&id) {
                view.can_go_back = can_back;
                view.can_go_forward = can_forward;
            }
        }
    }

    /// Reload current page
    pub async fn reload(&self, id: u64) -> Result<()> {
        let url = {
            let views = self.views.read().await;
            let view = views.get(&id).ok_or(WebViewError::NotInitialized)?;
            view.current_url.clone()
        };

        self.navigate_without_history(id, url).await
    }

    /// Stop loading
    pub async fn stop_loading(&self, id: u64) -> Result<()> {
        let mut views = self.views.write().await;
        let view = views.get_mut(&id).ok_or(WebViewError::NotInitialized)?;

        view.is_loading = false;
        view.load_state = LoadState::Idle;

        Ok(())
    }

    /// Execute JavaScript in the WebView
    pub async fn execute_js(&self, id: u64, script: String) -> Result<String> {
        let views = self.views.read().await;
        let _view = views.get(&id).ok_or(WebViewError::NotInitialized)?;

        let config = self.config.read().await;
        if !config.javascript_enabled {
            return Err(WebViewError::JsError("JavaScript is disabled".to_string()));
        }

        // In a real implementation, this would execute the script in the WebView
        // For now, return a placeholder response
        Ok(format!("Executed: {}", script))
    }

    /// Set page title
    pub async fn set_title(&self, id: u64, title: String) -> Result<()> {
        let mut views = self.views.write().await;
        let view = views.get_mut(&id).ok_or(WebViewError::NotInitialized)?;
        view.title = title;
        Ok(())
    }

    /// Get WebView state
    pub async fn get_state(&self, id: u64) -> Result<WebViewState> {
        let views = self.views.read().await;
        views
            .get(&id)
            .cloned()
            .ok_or(WebViewError::NotInitialized)
    }

    /// Set zoom level
    pub async fn set_zoom(&self, id: u64, level: f32) -> Result<()> {
        let mut views = self.views.write().await;
        let view = views.get_mut(&id).ok_or(WebViewError::NotInitialized)?;

        if level < 0.25 || level > 5.0 {
            return Err(WebViewError::JsError(
                "Zoom level must be between 0.25 and 5.0".to_string(),
            ));
        }

        view.zoom_level = level;
        Ok(())
    }

    /// Get zoom level
    pub async fn get_zoom(&self, id: u64) -> Result<f32> {
        let views = self.views.read().await;
        let view = views.get(&id).ok_or(WebViewError::NotInitialized)?;
        Ok(view.zoom_level)
    }

    /// Add resource to cache
    pub async fn cache_resource(&self, url: String, data: Vec<u8>, mime_type: String) {
        let resource = CachedResource {
            data,
            mime_type,
            cached_at: Utc::now(),
            expires_at: Some(Utc::now() + chrono::Duration::hours(24)),
        };

        let mut cache = self.cache.write().await;
        cache.insert(url, resource);

        // Clean up old cache entries if over size limit
        self.cleanup_cache().await;
    }

    /// Get resource from cache
    pub async fn get_cached_resource(&self, url: &str) -> Option<(Vec<u8>, String)> {
        let cache = self.cache.read().await;
        cache.get(url).and_then(|resource| {
            // Check if expired
            if let Some(expires) = resource.expires_at {
                if Utc::now() > expires {
                    return None;
                }
            }
            Some((resource.data.clone(), resource.mime_type.clone()))
        })
    }

    /// Clean up expired cache entries
    async fn cleanup_cache(&self) {
        let mut cache = self.cache.write().await;
        let now = Utc::now();

        cache.retain(|_, resource| {
            if let Some(expires) = resource.expires_at {
                now <= expires
            } else {
                true
            }
        });
    }

    /// Clear all cache
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    /// Get configuration
    pub async fn get_config(&self) -> WebViewConfig {
        self.config.read().await.clone()
    }

    /// Set configuration
    pub async fn set_config(&self, config: WebViewConfig) {
        let mut current = self.config.write().await;
        *current = config;
    }

    /// Get navigation events
    pub async fn get_navigation_events(&self) -> Vec<NavigationEvent> {
        self.navigation_events.read().await.clone()
    }

    /// Clear navigation events
    pub async fn clear_navigation_events(&self) {
        let mut events = self.navigation_events.write().await;
        events.clear();
    }

    /// Get all active WebView IDs
    pub async fn get_active_views(&self) -> Vec<u64> {
        let views = self.views.read().await;
        views.keys().copied().collect()
    }
}

impl Default for WebViewManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_webview() {
        let manager = WebViewManager::new();
        let id = manager.create_webview().await;
        assert!(id > 0);

        let state = manager.get_state(id).await.unwrap();
        assert_eq!(state.current_url, "about:blank");
        assert_eq!(state.load_state, LoadState::Idle);
    }

    #[tokio::test]
    async fn test_destroy_webview() {
        let manager = WebViewManager::new();
        let id = manager.create_webview().await;

        manager.destroy_webview(id).await.unwrap();

        let result = manager.get_state(id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_navigate() {
        let manager = WebViewManager::new();
        let id = manager.create_webview().await;

        manager
            .navigate(id, "https://example.com".to_string())
            .await
            .unwrap();

        let state = manager.get_state(id).await.unwrap();
        assert_eq!(state.current_url, "https://example.com");
        assert_eq!(state.load_state, LoadState::Complete);
    }

    #[tokio::test]
    async fn test_navigate_invalid_url() {
        let manager = WebViewManager::new();
        let id = manager.create_webview().await;

        let result = manager.navigate(id, "not a valid url".to_string()).await;
        assert!(matches!(result, Err(WebViewError::InvalidUrl(_))));
    }

    #[tokio::test]
    async fn test_history_navigation() {
        let manager = WebViewManager::new();
        let id = manager.create_webview().await;

        manager
            .navigate(id, "https://example.com".to_string())
            .await
            .unwrap();
        manager
            .navigate(id, "https://rust-lang.org".to_string())
            .await
            .unwrap();

        let state = manager.get_state(id).await.unwrap();
        assert!(state.can_go_back);
        assert!(!state.can_go_forward);

        manager.go_back(id).await.unwrap();
        let state = manager.get_state(id).await.unwrap();
        assert_eq!(state.current_url, "https://example.com");
        assert!(state.can_go_forward);

        manager.go_forward(id).await.unwrap();
        let state = manager.get_state(id).await.unwrap();
        assert_eq!(state.current_url, "https://rust-lang.org");
    }

    #[tokio::test]
    async fn test_reload() {
        let manager = WebViewManager::new();
        let id = manager.create_webview().await;

        manager
            .navigate(id, "https://example.com".to_string())
            .await
            .unwrap();

        manager.reload(id).await.unwrap();

        let state = manager.get_state(id).await.unwrap();
        assert_eq!(state.current_url, "https://example.com");
    }

    #[tokio::test]
    async fn test_stop_loading() {
        let manager = WebViewManager::new();
        let id = manager.create_webview().await;

        // Simulate loading state
        {
            let mut views = manager.views.write().await;
            if let Some(view) = views.get_mut(&id) {
                view.is_loading = true;
                view.load_state = LoadState::Loading;
            }
        }

        manager.stop_loading(id).await.unwrap();

        let state = manager.get_state(id).await.unwrap();
        assert!(!state.is_loading);
        assert_eq!(state.load_state, LoadState::Idle);
    }

    #[tokio::test]
    async fn test_execute_js() {
        let manager = WebViewManager::new();
        let id = manager.create_webview().await;

        let result = manager
            .execute_js(id, "console.log('test')".to_string())
            .await
            .unwrap();

        assert!(result.contains("console.log"));
    }

    #[tokio::test]
    async fn test_execute_js_disabled() {
        let manager = WebViewManager::new();
        let id = manager.create_webview().await;

        let mut config = manager.get_config().await;
        config.javascript_enabled = false;
        manager.set_config(config).await;

        let result = manager
            .execute_js(id, "console.log('test')".to_string())
            .await;

        assert!(matches!(result, Err(WebViewError::JsError(_))));
    }

    #[tokio::test]
    async fn test_set_title() {
        let manager = WebViewManager::new();
        let id = manager.create_webview().await;

        manager.set_title(id, "Custom Title".to_string()).await.unwrap();

        let state = manager.get_state(id).await.unwrap();
        assert_eq!(state.title, "Custom Title");
    }

    #[tokio::test]
    async fn test_zoom_level() {
        let manager = WebViewManager::new();
        let id = manager.create_webview().await;

        manager.set_zoom(id, 1.5).await.unwrap();
        let zoom = manager.get_zoom(id).await.unwrap();
        assert_eq!(zoom, 1.5);
    }

    #[tokio::test]
    async fn test_zoom_level_bounds() {
        let manager = WebViewManager::new();
        let id = manager.create_webview().await;

        let result = manager.set_zoom(id, 10.0).await;
        assert!(result.is_err());

        let result = manager.set_zoom(id, 0.1).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_cache_resource() {
        let manager = WebViewManager::new();

        manager
            .cache_resource(
                "https://example.com/image.png".to_string(),
                vec![1, 2, 3, 4, 5],
                "image/png".to_string(),
            )
            .await;

        let cached = manager
            .get_cached_resource("https://example.com/image.png")
            .await;

        assert!(cached.is_some());
        let (data, mime) = cached.unwrap();
        assert_eq!(data, vec![1, 2, 3, 4, 5]);
        assert_eq!(mime, "image/png");
    }

    #[tokio::test]
    async fn test_clear_cache() {
        let manager = WebViewManager::new();

        manager
            .cache_resource(
                "https://example.com/data".to_string(),
                vec![1, 2, 3],
                "application/octet-stream".to_string(),
            )
            .await;

        manager.clear_cache().await;

        let cached = manager
            .get_cached_resource("https://example.com/data")
            .await;

        assert!(cached.is_none());
    }

    #[tokio::test]
    async fn test_navigation_events() {
        let manager = WebViewManager::new();
        let id = manager.create_webview().await;

        manager
            .navigate(id, "https://example.com".to_string())
            .await
            .unwrap();

        let events = manager.get_navigation_events().await;
        assert!(events.len() >= 2); // At least Started and Completed

        manager.clear_navigation_events().await;
        let events = manager.get_navigation_events().await;
        assert!(events.is_empty());
    }

    #[tokio::test]
    async fn test_get_active_views() {
        let manager = WebViewManager::new();

        let id1 = manager.create_webview().await;
        let id2 = manager.create_webview().await;

        let active = manager.get_active_views().await;
        assert!(active.contains(&id1));
        assert!(active.contains(&id2));
        assert_eq!(active.len(), 2);
    }
}
