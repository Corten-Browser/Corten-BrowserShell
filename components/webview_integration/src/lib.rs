//! WebView Integration Component
//!
//! Provides web content rendering coordination using wry WebView.
//! Manages page loading, JavaScript bridge, resource caching,
//! navigation events, and egui embedding infrastructure.
//!
//! # Architecture
//!
//! The WebView integration consists of:
//! - `WebViewManager`: High-level state management (navigation, history, caching)
//! - `EmbeddedWebView`: Native WebView embedding within egui windows
//! - `WebViewBridge`: Message passing between egui and WebView
//!
//! # Embedding WebView in egui
//!
//! ```ignore
//! let mut webview = EmbeddedWebView::new(EmbedConfig::default());
//!
//! // In egui render loop
//! egui::CentralPanel::default().show(ctx, |ui| {
//!     webview.show(ui, ctx);
//! });
//! ```
//!
//! Note: Full wry integration requires system libraries (gtk, webkit2gtk on Linux).
//! The current implementation provides the API surface with placeholder rendering.

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

        {
            let mut cache = self.cache.write().await;
            cache.insert(url, resource);
        } // Drop the write lock before calling cleanup_cache

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

// ============================================================================
// WebView Embedding Infrastructure
// ============================================================================

/// Configuration for embedded WebView
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedConfig {
    /// Initial URL to load
    pub initial_url: String,
    /// Background color (RGBA)
    pub background_color: [u8; 4],
    /// Enable transparent background
    pub transparent: bool,
    /// Enable developer tools
    pub devtools_enabled: bool,
    /// Enable JavaScript
    pub javascript_enabled: bool,
    /// Custom user agent
    pub user_agent: Option<String>,
    /// Enable autoplay of media
    pub autoplay_enabled: bool,
    /// Enable clipboard access
    pub clipboard_enabled: bool,
    /// IPC handler name for JavaScript bridge
    pub ipc_handler_name: String,
    /// Custom initialization script
    pub initialization_script: Option<String>,
}

impl Default for EmbedConfig {
    fn default() -> Self {
        Self {
            initial_url: "about:blank".to_string(),
            background_color: [255, 255, 255, 255],
            transparent: false,
            devtools_enabled: false,
            javascript_enabled: true,
            user_agent: None,
            autoplay_enabled: false,
            clipboard_enabled: true,
            ipc_handler_name: "cortenIpc".to_string(),
            initialization_script: None,
        }
    }
}

/// Bounds for the WebView within the parent window
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WebViewBounds {
    /// X position in pixels
    pub x: i32,
    /// Y position in pixels
    pub y: i32,
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
}

impl Default for WebViewBounds {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            width: 800,
            height: 600,
        }
    }
}

/// Message types for the WebView bridge (egui <-> WebView communication)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BridgeMessage {
    /// Navigation request from egui
    Navigate { url: String },
    /// Go back in history
    GoBack,
    /// Go forward in history
    GoForward,
    /// Reload the page
    Reload,
    /// Stop loading
    StopLoading,
    /// Execute JavaScript
    ExecuteJs { script: String, callback_id: Option<String> },
    /// Set zoom level
    SetZoom { level: f32 },
    /// Toggle DevTools
    ToggleDevTools,
    /// Open DevTools
    OpenDevTools,
    /// Close DevTools
    CloseDevTools,
    /// Print page
    Print,
    /// Find in page
    Find { text: String, forward: bool, case_sensitive: bool },
    /// Clear find highlights
    ClearFind,
    /// Custom IPC message from JavaScript
    IpcMessage { method: String, params: serde_json::Value },
}

/// Events emitted by the WebView
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WebViewEvent {
    /// Page started loading
    NavigationStarted { url: String },
    /// Page committed (received first byte)
    NavigationCommitted { url: String },
    /// Page finished loading
    NavigationCompleted { url: String },
    /// Navigation failed
    NavigationFailed { url: String, error: String },
    /// Page title changed
    TitleChanged { title: String },
    /// Favicon changed
    FaviconChanged { url: Option<String> },
    /// JavaScript callback result
    JsResult { callback_id: String, result: serde_json::Value },
    /// JavaScript error
    JsError { callback_id: Option<String>, error: String },
    /// IPC message from JavaScript
    IpcReceived { method: String, params: serde_json::Value },
    /// WebView focused
    Focused,
    /// WebView blurred
    Blurred,
    /// New window requested (popup)
    NewWindowRequested { url: String },
    /// Download started
    DownloadStarted { url: String, suggested_filename: String },
    /// Context menu requested
    ContextMenuRequested { x: i32, y: i32 },
    /// DevTools opened
    DevToolsOpened,
    /// DevTools closed
    DevToolsClosed,
    /// Find result
    FindResult { active_match: u32, total_matches: u32 },
}

/// WebView bridge for message passing between egui and WebView
pub struct WebViewBridge {
    /// Pending messages to send to WebView
    outgoing: std::sync::Mutex<Vec<BridgeMessage>>,
    /// Events received from WebView
    incoming: std::sync::Mutex<Vec<WebViewEvent>>,
    /// JavaScript callback registry
    js_callbacks: std::sync::Mutex<HashMap<String, Box<dyn FnOnce(serde_json::Value) + Send>>>,
    /// Next callback ID
    next_callback_id: std::sync::atomic::AtomicU64,
}

impl WebViewBridge {
    /// Create a new WebView bridge
    pub fn new() -> Self {
        Self {
            outgoing: std::sync::Mutex::new(Vec::new()),
            incoming: std::sync::Mutex::new(Vec::new()),
            js_callbacks: std::sync::Mutex::new(HashMap::new()),
            next_callback_id: std::sync::atomic::AtomicU64::new(1),
        }
    }

    /// Send a message to the WebView
    pub fn send(&self, message: BridgeMessage) {
        if let Ok(mut outgoing) = self.outgoing.lock() {
            outgoing.push(message);
        }
    }

    /// Navigate to a URL
    pub fn navigate(&self, url: impl Into<String>) {
        self.send(BridgeMessage::Navigate { url: url.into() });
    }

    /// Go back in history
    pub fn go_back(&self) {
        self.send(BridgeMessage::GoBack);
    }

    /// Go forward in history
    pub fn go_forward(&self) {
        self.send(BridgeMessage::GoForward);
    }

    /// Reload the page
    pub fn reload(&self) {
        self.send(BridgeMessage::Reload);
    }

    /// Stop loading
    pub fn stop_loading(&self) {
        self.send(BridgeMessage::StopLoading);
    }

    /// Execute JavaScript with optional callback
    pub fn execute_js(&self, script: impl Into<String>) -> Option<String> {
        let callback_id = format!(
            "js_callback_{}",
            self.next_callback_id
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
        );
        self.send(BridgeMessage::ExecuteJs {
            script: script.into(),
            callback_id: Some(callback_id.clone()),
        });
        Some(callback_id)
    }

    /// Execute JavaScript without callback
    pub fn execute_js_fire_and_forget(&self, script: impl Into<String>) {
        self.send(BridgeMessage::ExecuteJs {
            script: script.into(),
            callback_id: None,
        });
    }

    /// Set zoom level (0.25 - 5.0)
    pub fn set_zoom(&self, level: f32) {
        self.send(BridgeMessage::SetZoom {
            level: level.clamp(0.25, 5.0),
        });
    }

    /// Toggle DevTools visibility
    pub fn toggle_devtools(&self) {
        self.send(BridgeMessage::ToggleDevTools);
    }

    /// Open DevTools
    pub fn open_devtools(&self) {
        self.send(BridgeMessage::OpenDevTools);
    }

    /// Close DevTools
    pub fn close_devtools(&self) {
        self.send(BridgeMessage::CloseDevTools);
    }

    /// Print the page
    pub fn print(&self) {
        self.send(BridgeMessage::Print);
    }

    /// Find text in page
    pub fn find(&self, text: impl Into<String>, forward: bool, case_sensitive: bool) {
        self.send(BridgeMessage::Find {
            text: text.into(),
            forward,
            case_sensitive,
        });
    }

    /// Clear find highlights
    pub fn clear_find(&self) {
        self.send(BridgeMessage::ClearFind);
    }

    /// Take all pending outgoing messages
    pub fn take_outgoing(&self) -> Vec<BridgeMessage> {
        if let Ok(mut outgoing) = self.outgoing.lock() {
            std::mem::take(&mut *outgoing)
        } else {
            Vec::new()
        }
    }

    /// Push an event from the WebView
    pub fn push_event(&self, event: WebViewEvent) {
        if let Ok(mut incoming) = self.incoming.lock() {
            incoming.push(event);
        }
    }

    /// Take all pending events
    pub fn take_events(&self) -> Vec<WebViewEvent> {
        if let Ok(mut incoming) = self.incoming.lock() {
            std::mem::take(&mut *incoming)
        } else {
            Vec::new()
        }
    }

    /// Poll for events (non-blocking)
    pub fn poll_events(&self) -> impl Iterator<Item = WebViewEvent> {
        self.take_events().into_iter()
    }
}

impl Default for WebViewBridge {
    fn default() -> Self {
        Self::new()
    }
}

/// State of the embedded WebView
#[derive(Debug, Clone)]
pub struct EmbeddedWebViewState {
    /// Current URL
    pub url: String,
    /// Page title
    pub title: String,
    /// Whether page is loading
    pub is_loading: bool,
    /// Can navigate back
    pub can_go_back: bool,
    /// Can navigate forward
    pub can_go_forward: bool,
    /// Current zoom level
    pub zoom_level: f32,
    /// DevTools are open
    pub devtools_open: bool,
    /// Current bounds
    pub bounds: WebViewBounds,
    /// WebView has focus
    pub has_focus: bool,
    /// Favicon URL
    pub favicon_url: Option<String>,
    /// Load progress (0.0 - 1.0)
    pub load_progress: f32,
}

impl Default for EmbeddedWebViewState {
    fn default() -> Self {
        Self {
            url: "about:blank".to_string(),
            title: "New Tab".to_string(),
            is_loading: false,
            can_go_back: false,
            can_go_forward: false,
            zoom_level: 1.0,
            devtools_open: false,
            bounds: WebViewBounds::default(),
            has_focus: false,
            favicon_url: None,
            load_progress: 0.0,
        }
    }
}

/// Embedded WebView for egui integration
///
/// This struct manages a WebView that can be embedded within an egui window.
/// Currently provides a placeholder implementation that renders a content area
/// with status information. Full wry WebView integration requires system
/// libraries (gtk, webkit2gtk on Linux, WebView2 on Windows).
///
/// # Usage
///
/// ```ignore
/// use webview_integration::{EmbeddedWebView, EmbedConfig};
///
/// let mut webview = EmbeddedWebView::new(EmbedConfig::default());
///
/// // Navigate to a URL
/// webview.navigate("https://example.com");
///
/// // In your egui update loop
/// egui::CentralPanel::default().show(ctx, |ui| {
///     webview.show(ui);
/// });
///
/// // Handle events
/// for event in webview.poll_events() {
///     match event {
///         WebViewEvent::TitleChanged { title } => {
///             // Update tab title
///         }
///         _ => {}
///     }
/// }
/// ```
pub struct EmbeddedWebView {
    /// Configuration
    config: EmbedConfig,
    /// Current state
    state: EmbeddedWebViewState,
    /// Communication bridge
    bridge: Arc<WebViewBridge>,
    /// Navigation history
    history: Vec<String>,
    /// Current history position
    history_position: usize,
    /// Pending JavaScript results
    js_results: HashMap<String, serde_json::Value>,
    /// Whether the native WebView is initialized
    native_initialized: bool,
    /// Last known bounds for resize detection
    last_bounds: Option<WebViewBounds>,
}

impl EmbeddedWebView {
    /// Create a new embedded WebView with the given configuration
    pub fn new(config: EmbedConfig) -> Self {
        let initial_url = config.initial_url.clone();
        Self {
            config,
            state: EmbeddedWebViewState {
                url: initial_url.clone(),
                ..Default::default()
            },
            bridge: Arc::new(WebViewBridge::new()),
            history: vec![initial_url],
            history_position: 0,
            js_results: HashMap::new(),
            native_initialized: false,
            last_bounds: None,
        }
    }

    /// Get a reference to the bridge for message passing
    pub fn bridge(&self) -> &Arc<WebViewBridge> {
        &self.bridge
    }

    /// Get current state
    pub fn state(&self) -> &EmbeddedWebViewState {
        &self.state
    }

    /// Navigate to a URL
    pub fn navigate(&mut self, url: impl Into<String>) {
        let url = url.into();

        // Update internal state
        self.state.is_loading = true;
        self.state.load_progress = 0.0;

        // Truncate forward history
        self.history.truncate(self.history_position + 1);
        self.history.push(url.clone());
        self.history_position = self.history.len() - 1;

        self.state.url = url.clone();
        self.update_navigation_state();

        // Send to bridge for native WebView
        self.bridge.navigate(url);

        // Emit navigation started event
        self.bridge.push_event(WebViewEvent::NavigationStarted {
            url: self.state.url.clone(),
        });
    }

    /// Go back in history
    pub fn go_back(&mut self) -> bool {
        if self.state.can_go_back && self.history_position > 0 {
            self.history_position -= 1;
            let url = self.history[self.history_position].clone();
            self.state.url = url.clone();
            self.state.is_loading = true;
            self.update_navigation_state();
            self.bridge.go_back();
            true
        } else {
            false
        }
    }

    /// Go forward in history
    pub fn go_forward(&mut self) -> bool {
        if self.state.can_go_forward && self.history_position < self.history.len() - 1 {
            self.history_position += 1;
            let url = self.history[self.history_position].clone();
            self.state.url = url.clone();
            self.state.is_loading = true;
            self.update_navigation_state();
            self.bridge.go_forward();
            true
        } else {
            false
        }
    }

    /// Reload the current page
    pub fn reload(&mut self) {
        self.state.is_loading = true;
        self.state.load_progress = 0.0;
        self.bridge.reload();
    }

    /// Stop loading
    pub fn stop_loading(&mut self) {
        self.state.is_loading = false;
        self.bridge.stop_loading();
    }

    /// Execute JavaScript
    pub fn execute_js(&mut self, script: impl Into<String>) -> Option<String> {
        self.bridge.execute_js(script)
    }

    /// Execute JavaScript without waiting for result
    pub fn execute_js_fire_and_forget(&mut self, script: impl Into<String>) {
        self.bridge.execute_js_fire_and_forget(script);
    }

    /// Inject JavaScript that runs on every page load
    pub fn inject_js(&mut self, script: impl Into<String>) {
        // In a real implementation, this would register a script to run on page load
        let script = script.into();
        if self.config.initialization_script.is_some() {
            self.config.initialization_script = Some(format!(
                "{}\n{}",
                self.config.initialization_script.as_ref().unwrap(),
                script
            ));
        } else {
            self.config.initialization_script = Some(script);
        }
    }

    /// Set zoom level
    pub fn set_zoom(&mut self, level: f32) {
        let level = level.clamp(0.25, 5.0);
        self.state.zoom_level = level;
        self.bridge.set_zoom(level);
    }

    /// Get zoom level
    pub fn zoom_level(&self) -> f32 {
        self.state.zoom_level
    }

    /// Toggle DevTools
    pub fn toggle_devtools(&mut self) {
        self.state.devtools_open = !self.state.devtools_open;
        self.bridge.toggle_devtools();
        if self.state.devtools_open {
            self.bridge.push_event(WebViewEvent::DevToolsOpened);
        } else {
            self.bridge.push_event(WebViewEvent::DevToolsClosed);
        }
    }

    /// Open DevTools
    pub fn open_devtools(&mut self) {
        if !self.state.devtools_open {
            self.state.devtools_open = true;
            self.bridge.open_devtools();
            self.bridge.push_event(WebViewEvent::DevToolsOpened);
        }
    }

    /// Close DevTools
    pub fn close_devtools(&mut self) {
        if self.state.devtools_open {
            self.state.devtools_open = false;
            self.bridge.close_devtools();
            self.bridge.push_event(WebViewEvent::DevToolsClosed);
        }
    }

    /// Check if DevTools are enabled
    pub fn devtools_enabled(&self) -> bool {
        self.config.devtools_enabled
    }

    /// Check if DevTools are open
    pub fn devtools_open(&self) -> bool {
        self.state.devtools_open
    }

    /// Find text in page
    pub fn find(&self, text: impl Into<String>, forward: bool, case_sensitive: bool) {
        self.bridge.find(text, forward, case_sensitive);
    }

    /// Clear find highlights
    pub fn clear_find(&self) {
        self.bridge.clear_find();
    }

    /// Print the page
    pub fn print(&self) {
        self.bridge.print();
    }

    /// Update bounds (called when container resizes)
    pub fn set_bounds(&mut self, bounds: WebViewBounds) {
        let bounds_changed = self.last_bounds.map_or(true, |last| last != bounds);

        if bounds_changed {
            self.state.bounds = bounds;
            self.last_bounds = Some(bounds);

            // In a real implementation, this would resize the native WebView
            // wry::WebView::set_bounds() or similar
        }
    }

    /// Get current bounds
    pub fn bounds(&self) -> &WebViewBounds {
        &self.state.bounds
    }

    /// Set focus
    pub fn focus(&mut self) {
        if !self.state.has_focus {
            self.state.has_focus = true;
            self.bridge.push_event(WebViewEvent::Focused);
        }
    }

    /// Remove focus
    pub fn blur(&mut self) {
        if self.state.has_focus {
            self.state.has_focus = false;
            self.bridge.push_event(WebViewEvent::Blurred);
        }
    }

    /// Check if WebView has focus
    pub fn has_focus(&self) -> bool {
        self.state.has_focus
    }

    /// Poll for events
    pub fn poll_events(&self) -> impl Iterator<Item = WebViewEvent> {
        self.bridge.poll_events()
    }

    /// Process pending bridge messages
    pub fn process_messages(&mut self) {
        for message in self.bridge.take_outgoing() {
            // In a real implementation, this would send messages to the native WebView
            // For now, we simulate the behavior
            match message {
                BridgeMessage::Navigate { url } => {
                    // Simulate navigation completion after a short delay
                    self.state.is_loading = true;
                    self.state.url = url.clone();

                    // In placeholder mode, immediately complete
                    self.state.is_loading = false;
                    self.state.load_progress = 1.0;
                    self.bridge.push_event(WebViewEvent::NavigationCompleted { url });
                }
                BridgeMessage::SetZoom { level } => {
                    self.state.zoom_level = level;
                }
                _ => {
                    // Other messages would be forwarded to native WebView
                }
            }
        }
    }

    /// Update can_go_back and can_go_forward state
    fn update_navigation_state(&mut self) {
        self.state.can_go_back = self.history_position > 0;
        self.state.can_go_forward = self.history_position < self.history.len() - 1;
    }

    /// Render the WebView in an egui UI
    ///
    /// This should be called from within an egui panel or area.
    /// Returns the rect where the WebView should be rendered.
    pub fn show(&mut self, ui: &mut egui::Ui) -> egui::Response {
        // Process any pending messages
        self.process_messages();

        // Get available rect and update bounds
        let available = ui.available_rect_before_wrap();
        let bounds = WebViewBounds {
            x: available.min.x as i32,
            y: available.min.y as i32,
            width: available.width() as u32,
            height: available.height() as u32,
        };
        self.set_bounds(bounds);

        // Render placeholder content (real implementation would composite native WebView)
        let (rect, response) = ui.allocate_exact_size(available.size(), egui::Sense::click_and_drag());

        if ui.is_rect_visible(rect) {
            // Draw background
            let bg_color = egui::Color32::from_rgba_unmultiplied(
                self.config.background_color[0],
                self.config.background_color[1],
                self.config.background_color[2],
                self.config.background_color[3],
            );
            ui.painter().rect_filled(rect, 0.0, bg_color);

            // Draw placeholder content
            let center = rect.center();

            // Show URL
            ui.painter().text(
                center - egui::vec2(0.0, 40.0),
                egui::Align2::CENTER_CENTER,
                format!("URL: {}", self.state.url),
                egui::FontId::default(),
                egui::Color32::GRAY,
            );

            // Show loading state
            let status = if self.state.is_loading {
                format!("Loading... {:.0}%", self.state.load_progress * 100.0)
            } else {
                "Ready".to_string()
            };
            ui.painter().text(
                center,
                egui::Align2::CENTER_CENTER,
                status,
                egui::FontId::default(),
                egui::Color32::DARK_GRAY,
            );

            // Show title
            ui.painter().text(
                center + egui::vec2(0.0, 40.0),
                egui::Align2::CENTER_CENTER,
                format!("Title: {}", self.state.title),
                egui::FontId::default(),
                egui::Color32::GRAY,
            );

            // Draw border
            ui.painter().rect_stroke(rect, 0.0, egui::Stroke::new(1.0, egui::Color32::DARK_GRAY));

            // Info about placeholder mode
            ui.painter().text(
                egui::pos2(rect.min.x + 10.0, rect.max.y - 20.0),
                egui::Align2::LEFT_CENTER,
                "(WebView placeholder - wry integration pending)",
                egui::FontId::proportional(10.0),
                egui::Color32::from_gray(128),
            );
        }

        // Handle focus
        if response.clicked() {
            self.focus();
        }

        response
    }

    /// Show with custom navigation bar
    pub fn show_with_navigation(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let mut navigation_event: Option<NavigationAction> = None;

        // Navigation toolbar
        ui.horizontal(|ui| {
            // Back button
            let back_btn = ui.add_enabled(self.state.can_go_back, egui::Button::new("<"));
            if back_btn.clicked() {
                navigation_event = Some(NavigationAction::Back);
            }

            // Forward button
            let fwd_btn = ui.add_enabled(self.state.can_go_forward, egui::Button::new(">"));
            if fwd_btn.clicked() {
                navigation_event = Some(NavigationAction::Forward);
            }

            // Reload/Stop button
            if self.state.is_loading {
                if ui.button("X").clicked() {
                    navigation_event = Some(NavigationAction::Stop);
                }
            } else {
                if ui.button("\u{21BB}").clicked() {
                    navigation_event = Some(NavigationAction::Reload);
                }
            }

            // URL display
            ui.label(&self.state.url);

            // Zoom controls
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("+").clicked() {
                    self.set_zoom(self.state.zoom_level + 0.1);
                }
                ui.label(format!("{:.0}%", self.state.zoom_level * 100.0));
                if ui.small_button("-").clicked() {
                    self.set_zoom(self.state.zoom_level - 0.1);
                }

                // DevTools button
                if self.config.devtools_enabled {
                    let devtools_text = if self.state.devtools_open { "[X]" } else { "{ }" };
                    if ui.small_button(devtools_text).on_hover_text("Toggle DevTools").clicked() {
                        self.toggle_devtools();
                    }
                }
            });
        });

        // Process navigation events
        match navigation_event {
            Some(NavigationAction::Back) => { self.go_back(); }
            Some(NavigationAction::Forward) => { self.go_forward(); }
            Some(NavigationAction::Reload) => { self.reload(); }
            Some(NavigationAction::Stop) => { self.stop_loading(); }
            None => {}
        }

        // Show WebView content
        self.show(ui)
    }
}

/// Navigation action enum (internal use)
enum NavigationAction {
    Back,
    Forward,
    Reload,
    Stop,
}

impl Default for EmbeddedWebView {
    fn default() -> Self {
        Self::new(EmbedConfig::default())
    }
}

// ============================================================================
// Native Window Handle Integration (for future wry integration)
// ============================================================================

/// Window handle types for native WebView embedding
#[derive(Debug, Clone)]
pub enum WindowHandle {
    /// Windows HWND
    #[cfg(target_os = "windows")]
    Windows(isize),
    /// macOS NSView
    #[cfg(target_os = "macos")]
    MacOS(*mut std::ffi::c_void),
    /// Linux X11 window ID
    #[cfg(target_os = "linux")]
    X11(u64),
    /// Linux Wayland surface
    #[cfg(target_os = "linux")]
    Wayland(*mut std::ffi::c_void),
    /// Placeholder for unsupported platforms
    Unsupported,
}

impl Default for WindowHandle {
    fn default() -> Self {
        Self::Unsupported
    }
}

// Safety: WindowHandle pointers are only used for passing to native APIs
unsafe impl Send for WindowHandle {}
unsafe impl Sync for WindowHandle {}

/// Builder for creating native WebViews
///
/// This is a placeholder for the wry::WebViewBuilder integration.
/// When wry is enabled, this will wrap the actual builder.
pub struct WebViewBuilder {
    config: EmbedConfig,
    bounds: WebViewBounds,
    window_handle: Option<WindowHandle>,
}

impl WebViewBuilder {
    /// Create a new WebView builder
    pub fn new() -> Self {
        Self {
            config: EmbedConfig::default(),
            bounds: WebViewBounds::default(),
            window_handle: None,
        }
    }

    /// Set the configuration
    pub fn with_config(mut self, config: EmbedConfig) -> Self {
        self.config = config;
        self
    }

    /// Set the bounds
    pub fn with_bounds(mut self, bounds: WebViewBounds) -> Self {
        self.bounds = bounds;
        self
    }

    /// Set the parent window handle
    pub fn with_window_handle(mut self, handle: WindowHandle) -> Self {
        self.window_handle = Some(handle);
        self
    }

    /// Set initial URL
    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.config.initial_url = url.into();
        self
    }

    /// Enable DevTools
    pub fn with_devtools(mut self, enabled: bool) -> Self {
        self.config.devtools_enabled = enabled;
        self
    }

    /// Enable/disable JavaScript
    pub fn with_javascript(mut self, enabled: bool) -> Self {
        self.config.javascript_enabled = enabled;
        self
    }

    /// Set transparent background
    pub fn with_transparent(mut self, transparent: bool) -> Self {
        self.config.transparent = transparent;
        self
    }

    /// Set background color
    pub fn with_background_color(mut self, r: u8, g: u8, b: u8, a: u8) -> Self {
        self.config.background_color = [r, g, b, a];
        self
    }

    /// Set user agent
    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.config.user_agent = Some(user_agent.into());
        self
    }

    /// Set initialization script
    pub fn with_init_script(mut self, script: impl Into<String>) -> Self {
        self.config.initialization_script = Some(script.into());
        self
    }

    /// Build the embedded WebView
    ///
    /// Note: This currently returns a placeholder implementation.
    /// Full wry integration will be added when system libraries are available.
    pub fn build(self) -> Result<EmbeddedWebView> {
        let mut webview = EmbeddedWebView::new(self.config);
        webview.set_bounds(self.bounds);

        // In a real implementation with wry:
        // - Create wry::WebView with the window handle
        // - Set up IPC handlers
        // - Configure navigation callbacks

        Ok(webview)
    }
}

impl Default for WebViewBuilder {
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

    // ====================================================================
    // Tests for Embedding Infrastructure
    // ====================================================================

    #[test]
    fn test_embed_config_default() {
        let config = EmbedConfig::default();
        assert_eq!(config.initial_url, "about:blank");
        assert!(config.javascript_enabled);
        assert!(!config.devtools_enabled);
        assert_eq!(config.background_color, [255, 255, 255, 255]);
        assert_eq!(config.ipc_handler_name, "cortenIpc");
    }

    #[test]
    fn test_webview_bounds_default() {
        let bounds = WebViewBounds::default();
        assert_eq!(bounds.x, 0);
        assert_eq!(bounds.y, 0);
        assert_eq!(bounds.width, 800);
        assert_eq!(bounds.height, 600);
    }

    #[test]
    fn test_embedded_webview_creation() {
        let config = EmbedConfig {
            initial_url: "https://example.com".to_string(),
            devtools_enabled: true,
            ..Default::default()
        };
        let webview = EmbeddedWebView::new(config);

        assert_eq!(webview.state().url, "https://example.com");
        assert!(!webview.state().is_loading);
        assert!(!webview.state().can_go_back);
        assert!(!webview.state().can_go_forward);
        assert!(webview.devtools_enabled());
    }

    #[test]
    fn test_embedded_webview_navigate() {
        let mut webview = EmbeddedWebView::default();

        webview.navigate("https://rust-lang.org");

        assert_eq!(webview.state().url, "https://rust-lang.org");
        assert!(webview.state().is_loading);
    }

    #[test]
    fn test_embedded_webview_history() {
        let mut webview = EmbeddedWebView::default();

        // Initial state: history = ["about:blank"], position = 0
        assert!(!webview.state().can_go_back);
        assert!(!webview.state().can_go_forward);

        // Navigate to first page: history = ["about:blank", "example.com"], position = 1
        webview.navigate("https://example.com");
        webview.process_messages(); // Simulate completion
        assert!(webview.state().can_go_back); // Can go back to about:blank
        assert!(!webview.state().can_go_forward);

        // Navigate to second page: history = ["about:blank", "example.com", "rust-lang.org"], position = 2
        webview.navigate("https://rust-lang.org");
        webview.process_messages();
        assert!(webview.state().can_go_back);
        assert!(!webview.state().can_go_forward);

        // Go back: position = 1
        assert!(webview.go_back());
        assert_eq!(webview.state().url, "https://example.com");
        assert!(webview.state().can_go_forward);
        assert!(webview.state().can_go_back); // Can still go back to about:blank

        // Go back again: position = 0
        assert!(webview.go_back());
        assert_eq!(webview.state().url, "about:blank");
        assert!(webview.state().can_go_forward);
        assert!(!webview.state().can_go_back); // Now at start

        // Go forward twice to get back to rust-lang.org
        assert!(webview.go_forward());
        assert_eq!(webview.state().url, "https://example.com");
        assert!(webview.go_forward());
        assert_eq!(webview.state().url, "https://rust-lang.org");
        assert!(!webview.state().can_go_forward);
    }

    #[test]
    fn test_embedded_webview_cant_go_back_at_start() {
        let mut webview = EmbeddedWebView::default();
        assert!(!webview.go_back());
    }

    #[test]
    fn test_embedded_webview_cant_go_forward_at_end() {
        let mut webview = EmbeddedWebView::default();
        webview.navigate("https://example.com");
        assert!(!webview.go_forward());
    }

    #[test]
    fn test_embedded_webview_zoom() {
        let mut webview = EmbeddedWebView::default();

        webview.set_zoom(1.5);
        assert_eq!(webview.zoom_level(), 1.5);

        // Test clamping
        webview.set_zoom(10.0);
        assert_eq!(webview.zoom_level(), 5.0);

        webview.set_zoom(0.1);
        assert_eq!(webview.zoom_level(), 0.25);
    }

    #[test]
    fn test_embedded_webview_devtools() {
        let config = EmbedConfig {
            devtools_enabled: true,
            ..Default::default()
        };
        let mut webview = EmbeddedWebView::new(config);

        assert!(!webview.devtools_open());

        webview.open_devtools();
        assert!(webview.devtools_open());

        webview.close_devtools();
        assert!(!webview.devtools_open());

        webview.toggle_devtools();
        assert!(webview.devtools_open());

        webview.toggle_devtools();
        assert!(!webview.devtools_open());
    }

    #[test]
    fn test_embedded_webview_focus() {
        let mut webview = EmbeddedWebView::default();

        assert!(!webview.has_focus());

        webview.focus();
        assert!(webview.has_focus());

        webview.blur();
        assert!(!webview.has_focus());
    }

    #[test]
    fn test_embedded_webview_bounds() {
        let mut webview = EmbeddedWebView::default();

        let bounds = WebViewBounds {
            x: 100,
            y: 200,
            width: 1024,
            height: 768,
        };
        webview.set_bounds(bounds);

        assert_eq!(webview.bounds().x, 100);
        assert_eq!(webview.bounds().y, 200);
        assert_eq!(webview.bounds().width, 1024);
        assert_eq!(webview.bounds().height, 768);
    }

    #[test]
    fn test_embedded_webview_reload_stop() {
        let mut webview = EmbeddedWebView::default();
        webview.navigate("https://example.com");

        webview.reload();
        assert!(webview.state().is_loading);

        webview.stop_loading();
        assert!(!webview.state().is_loading);
    }

    #[test]
    fn test_embedded_webview_execute_js() {
        let mut webview = EmbeddedWebView::default();

        let callback_id = webview.execute_js("document.title");
        assert!(callback_id.is_some());

        // Fire and forget doesn't return callback
        webview.execute_js_fire_and_forget("console.log('test')");
    }

    #[test]
    fn test_embedded_webview_inject_js() {
        let mut webview = EmbeddedWebView::default();

        webview.inject_js("window.corten = {};");
        assert!(webview.config.initialization_script.is_some());

        webview.inject_js("window.corten.version = '0.5.0';");
        let script = webview.config.initialization_script.as_ref().unwrap();
        assert!(script.contains("window.corten = {};"));
        assert!(script.contains("window.corten.version = '0.5.0';"));
    }

    #[test]
    fn test_webview_bridge_messaging() {
        let bridge = WebViewBridge::new();

        // Send messages
        bridge.navigate("https://example.com");
        bridge.go_back();
        bridge.reload();

        // Take messages
        let messages = bridge.take_outgoing();
        assert_eq!(messages.len(), 3);
        assert!(matches!(messages[0], BridgeMessage::Navigate { .. }));
        assert!(matches!(messages[1], BridgeMessage::GoBack));
        assert!(matches!(messages[2], BridgeMessage::Reload));

        // Messages should be cleared
        let messages = bridge.take_outgoing();
        assert!(messages.is_empty());
    }

    #[test]
    fn test_webview_bridge_events() {
        let bridge = WebViewBridge::new();

        // Push events
        bridge.push_event(WebViewEvent::NavigationStarted {
            url: "https://example.com".to_string(),
        });
        bridge.push_event(WebViewEvent::TitleChanged {
            title: "Example".to_string(),
        });

        // Take events
        let events: Vec<_> = bridge.poll_events().collect();
        assert_eq!(events.len(), 2);

        // Events should be cleared
        let events: Vec<_> = bridge.poll_events().collect();
        assert!(events.is_empty());
    }

    #[test]
    fn test_webview_bridge_js_callback() {
        let bridge = WebViewBridge::new();

        let callback_id1 = bridge.execute_js("script1");
        let callback_id2 = bridge.execute_js("script2");

        assert!(callback_id1.is_some());
        assert!(callback_id2.is_some());
        assert_ne!(callback_id1, callback_id2);
    }

    #[test]
    fn test_webview_builder() {
        let webview = WebViewBuilder::new()
            .with_url("https://example.com")
            .with_devtools(true)
            .with_javascript(true)
            .with_transparent(false)
            .with_background_color(240, 240, 240, 255)
            .with_user_agent("TestBrowser/1.0")
            .with_bounds(WebViewBounds {
                x: 0,
                y: 0,
                width: 1920,
                height: 1080,
            })
            .build()
            .unwrap();

        assert_eq!(webview.state().url, "https://example.com");
        assert!(webview.devtools_enabled());
        assert_eq!(webview.bounds().width, 1920);
        assert_eq!(webview.bounds().height, 1080);
    }

    #[test]
    fn test_webview_builder_with_init_script() {
        let webview = WebViewBuilder::new()
            .with_init_script("window.initialized = true;")
            .build()
            .unwrap();

        assert!(webview.config.initialization_script.is_some());
    }

    #[test]
    fn test_embedded_webview_state_default() {
        let state = EmbeddedWebViewState::default();
        assert_eq!(state.url, "about:blank");
        assert_eq!(state.title, "New Tab");
        assert!(!state.is_loading);
        assert!(!state.can_go_back);
        assert!(!state.can_go_forward);
        assert_eq!(state.zoom_level, 1.0);
        assert!(!state.devtools_open);
        assert!(!state.has_focus);
        assert_eq!(state.load_progress, 0.0);
    }

    #[test]
    fn test_bridge_message_variants() {
        // Test all bridge message variants can be created
        let messages = vec![
            BridgeMessage::Navigate { url: "test".to_string() },
            BridgeMessage::GoBack,
            BridgeMessage::GoForward,
            BridgeMessage::Reload,
            BridgeMessage::StopLoading,
            BridgeMessage::ExecuteJs { script: "test".to_string(), callback_id: None },
            BridgeMessage::SetZoom { level: 1.0 },
            BridgeMessage::ToggleDevTools,
            BridgeMessage::OpenDevTools,
            BridgeMessage::CloseDevTools,
            BridgeMessage::Print,
            BridgeMessage::Find { text: "test".to_string(), forward: true, case_sensitive: false },
            BridgeMessage::ClearFind,
            BridgeMessage::IpcMessage { method: "test".to_string(), params: serde_json::Value::Null },
        ];
        assert_eq!(messages.len(), 14);
    }

    #[test]
    fn test_webview_event_variants() {
        // Test all event variants can be created
        let events = vec![
            WebViewEvent::NavigationStarted { url: "test".to_string() },
            WebViewEvent::NavigationCommitted { url: "test".to_string() },
            WebViewEvent::NavigationCompleted { url: "test".to_string() },
            WebViewEvent::NavigationFailed { url: "test".to_string(), error: "error".to_string() },
            WebViewEvent::TitleChanged { title: "test".to_string() },
            WebViewEvent::FaviconChanged { url: Some("test".to_string()) },
            WebViewEvent::JsResult { callback_id: "1".to_string(), result: serde_json::Value::Null },
            WebViewEvent::JsError { callback_id: None, error: "error".to_string() },
            WebViewEvent::IpcReceived { method: "test".to_string(), params: serde_json::Value::Null },
            WebViewEvent::Focused,
            WebViewEvent::Blurred,
            WebViewEvent::NewWindowRequested { url: "test".to_string() },
            WebViewEvent::DownloadStarted { url: "test".to_string(), suggested_filename: "file.txt".to_string() },
            WebViewEvent::ContextMenuRequested { x: 0, y: 0 },
            WebViewEvent::DevToolsOpened,
            WebViewEvent::DevToolsClosed,
            WebViewEvent::FindResult { active_match: 1, total_matches: 5 },
        ];
        assert_eq!(events.len(), 17);
    }

    #[test]
    fn test_window_handle_default() {
        let handle = WindowHandle::default();
        assert!(matches!(handle, WindowHandle::Unsupported));
    }
}
