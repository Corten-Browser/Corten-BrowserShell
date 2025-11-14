//! Browser Shell Component
//!
//! Main orchestrator coordinating all browser shell components through the message bus.
//!
//! This component acts as the integration layer, managing:
//! - Window lifecycle (via window_manager)
//! - Tab lifecycle (via tab_manager)
//! - UI state (via ui_chrome)
//! - User settings (via settings_manager)
//! - Downloads (via downloads_manager)
//! - Bookmarks (via bookmarks_manager)
//! - Component communication (via message_bus)

use shared_types::{ComponentError, TabId, WindowConfig, WindowId};
use std::sync::Arc;
use tokio::sync::RwLock;

// Re-export public types
pub use shared_types;

/// Browser shell configuration
#[derive(Debug, Clone)]
pub struct ShellConfig {
    /// Window configuration
    pub window_config: WindowConfig,
    /// Enable developer tools
    pub enable_devtools: bool,
    /// Enable extensions
    pub enable_extensions: bool,
    /// User data directory path
    pub user_data_dir: String,
}

/// Main browser shell orchestrator
///
/// Coordinates all browser components and manages application state.
pub struct BrowserShell {
    /// Message bus for component communication
    message_bus: Option<Arc<message_bus::MessageBus>>,

    /// Window manager (generic over platform window type)
    #[cfg(target_os = "linux")]
    window_manager:
        Option<Arc<RwLock<window_manager::WindowManager<platform_abstraction::LinuxWindow>>>>,

    #[cfg(target_os = "windows")]
    window_manager:
        Option<Arc<RwLock<window_manager::WindowManager<platform_abstraction::WindowsWindow>>>>,

    #[cfg(target_os = "macos")]
    window_manager:
        Option<Arc<RwLock<window_manager::WindowManager<platform_abstraction::MacWindow>>>>,

    /// Tab manager
    tab_manager: Option<Arc<RwLock<tab_manager::TabManager>>>,

    /// UI chrome manager
    ui_chrome: Option<Arc<RwLock<ui_chrome::UiChrome>>>,

    /// Settings manager
    settings_manager: Option<Arc<RwLock<settings_manager::SettingsManager>>>,

    /// Downloads manager
    downloads_manager: Option<Arc<RwLock<downloads_manager::DownloadsManager>>>,

    /// Bookmarks manager
    bookmarks_manager: Option<Arc<RwLock<bookmarks_manager::BookmarksManager>>>,

    /// Active window ID
    active_window: Option<WindowId>,

    /// Active tab ID
    active_tab: Option<TabId>,

    /// Configuration
    config: Option<ShellConfig>,
}

impl BrowserShell {
    /// Create a new browser shell instance
    pub fn new() -> Self {
        Self {
            message_bus: None,
            window_manager: None,
            tab_manager: None,
            ui_chrome: None,
            settings_manager: None,
            downloads_manager: None,
            bookmarks_manager: None,
            active_window: None,
            active_tab: None,
            config: None,
        }
    }

    /// Initialize browser shell and all components
    ///
    /// This method:
    /// 1. Creates the message bus
    /// 2. Initializes all component managers
    /// 3. Registers components with the message bus
    /// 4. Loads user settings
    pub async fn initialize(&mut self, config: ShellConfig) -> Result<(), ComponentError> {
        // Store configuration
        self.config = Some(config.clone());

        // Initialize message bus (handles Result)
        let message_bus = Arc::new(message_bus::MessageBus::new()?);
        self.message_bus = Some(message_bus.clone());

        // Initialize window manager
        let window_manager = window_manager::WindowManager::new();
        self.window_manager = Some(Arc::new(RwLock::new(window_manager)));

        // Initialize tab manager
        let tab_manager = tab_manager::TabManager::new();
        self.tab_manager = Some(Arc::new(RwLock::new(tab_manager)));

        // Initialize UI chrome
        let ui_chrome = ui_chrome::UiChrome::new();
        self.ui_chrome = Some(Arc::new(RwLock::new(ui_chrome)));

        // Initialize settings manager with custom config directory
        let settings_manager = settings_manager::SettingsManager::with_config_dir(
            std::path::PathBuf::from(&config.user_data_dir),
        );
        // Load settings from disk (if file exists)
        settings_manager
            .load()
            .await
            .map_err(|e| ComponentError::InitializationFailed(e.to_string()))?;
        self.settings_manager = Some(Arc::new(RwLock::new(settings_manager)));

        // Initialize downloads manager
        let downloads_manager = downloads_manager::DownloadsManager::new();
        self.downloads_manager = Some(Arc::new(RwLock::new(downloads_manager)));

        // Initialize bookmarks manager (load is a static function)
        let bookmarks_manager = bookmarks_manager::BookmarksManager::load(
            std::path::PathBuf::from(&config.user_data_dir),
        )
        .await
        .map_err(|e| ComponentError::InitializationFailed(e.to_string()))?;
        self.bookmarks_manager = Some(Arc::new(RwLock::new(bookmarks_manager)));

        Ok(())
    }

    /// Gracefully shutdown all components
    pub async fn shutdown(&mut self) -> Result<(), ComponentError> {
        // Save settings (no arguments needed)
        if let Some(settings_manager) = &self.settings_manager {
            settings_manager.write().await.save().await.map_err(|e| {
                ComponentError::InvalidState(format!("Failed to save settings: {}", e))
            })?;
        }

        // Save bookmarks (no arguments needed)
        if let Some(bookmarks_manager) = &self.bookmarks_manager {
            bookmarks_manager.read().await.save().await.map_err(|e| {
                ComponentError::InvalidState(format!("Failed to save bookmarks: {}", e))
            })?;
        }

        // Clear all managers
        self.window_manager = None;
        self.tab_manager = None;
        self.ui_chrome = None;
        self.settings_manager = None;
        self.downloads_manager = None;
        self.bookmarks_manager = None;
        self.message_bus = None;

        Ok(())
    }

    /// Start the browser shell event loop
    ///
    /// For Phase 1, this is a stub implementation that just logs and waits.
    /// Full egui integration will come in later phases.
    pub async fn run(&mut self) -> Result<(), ComponentError> {
        println!("Browser shell running...");

        // Stub event loop - in real implementation this would be egui event loop
        // For now, just wait indefinitely
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            // In production, this would:
            // - Handle window events
            // - Process message bus messages
            // - Update UI state
            // - Handle user input
            // Break condition would be shutdown signal
        }
    }

    /// Create a new browser window
    pub async fn new_window(
        &mut self,
        config: Option<WindowConfig>,
    ) -> Result<WindowId, ComponentError> {
        let window_manager = self.window_manager.as_ref().ok_or_else(|| {
            ComponentError::InvalidState("Window manager not initialized".to_string())
        })?;

        let window_config = config.unwrap_or_else(|| {
            self.config
                .as_ref()
                .map(|c| c.window_config.clone())
                .unwrap_or_default()
        });

        let mut wm = window_manager.write().await;

        // WindowManager creates the platform window internally
        let window_id = wm
            .create_window(window_config)
            .await
            .map_err(|e| ComponentError::InvalidState(format!("Failed to create window: {}", e)))?;

        // Set as active window
        self.active_window = Some(window_id);

        Ok(window_id)
    }

    /// Create a new tab in the active window
    pub async fn new_tab(&mut self, url: Option<String>) -> Result<TabId, ComponentError> {
        let window_id = self
            .active_window
            .ok_or_else(|| ComponentError::InvalidState("No active window".to_string()))?;

        let tab_manager = self.tab_manager.as_ref().ok_or_else(|| {
            ComponentError::InvalidState("Tab manager not initialized".to_string())
        })?;

        let mut tm = tab_manager.write().await;

        // tab_manager expects Option<String> directly
        let tab_id = tm
            .create_tab(window_id, url)
            .await
            .map_err(|e| ComponentError::InvalidState(format!("Failed to create tab: {}", e)))?;

        // Set as active tab
        self.active_tab = Some(tab_id);

        // Note: Tab-to-window association is managed by tab_manager
        // window_manager doesn't expose tab management API

        Ok(tab_id)
    }

    /// Navigate the active tab to a URL
    pub async fn navigate(&mut self, url: String) -> Result<(), ComponentError> {
        let tab_id = self
            .active_tab
            .ok_or_else(|| ComponentError::InvalidState("No active tab".to_string()))?;

        let tab_manager = self.tab_manager.as_ref().ok_or_else(|| {
            ComponentError::InvalidState("Tab manager not initialized".to_string())
        })?;

        // tab_manager expects String directly
        let mut tm = tab_manager.write().await;
        tm.navigate(tab_id, url)
            .await
            .map_err(|e| ComponentError::InvalidState(format!("Failed to navigate: {}", e)))?;

        Ok(())
    }

    /// Get the active window ID
    pub fn active_window(&self) -> Option<WindowId> {
        self.active_window
    }

    /// Get the active tab ID
    pub fn active_tab(&self) -> Option<TabId> {
        self.active_tab
    }
}

impl Default for BrowserShell {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_new_browser_shell() {
        let shell = BrowserShell::new();
        assert!(shell.message_bus.is_none());
        assert!(shell.window_manager.is_none());
        assert!(shell.active_window.is_none());
    }

    #[tokio::test]
    async fn test_default_browser_shell() {
        let shell = BrowserShell::default();
        assert!(shell.message_bus.is_none());
    }

    #[tokio::test]
    async fn test_initialize_succeeds() {
        let temp_dir = TempDir::new().unwrap();
        let config = ShellConfig {
            window_config: WindowConfig::default(),
            enable_devtools: true,
            enable_extensions: false,
            user_data_dir: temp_dir.path().to_str().unwrap().to_string(),
        };

        let mut shell = BrowserShell::new();
        let result = shell.initialize(config).await;

        assert!(result.is_ok(), "Initialization should succeed");
        assert!(shell.message_bus.is_some());
        assert!(shell.window_manager.is_some());
        assert!(shell.tab_manager.is_some());
        assert!(shell.ui_chrome.is_some());
        assert!(shell.settings_manager.is_some());
        assert!(shell.downloads_manager.is_some());
        assert!(shell.bookmarks_manager.is_some());
    }

    #[tokio::test]
    async fn test_shutdown_succeeds() {
        let temp_dir = TempDir::new().unwrap();
        let config = ShellConfig {
            window_config: WindowConfig::default(),
            enable_devtools: true,
            enable_extensions: false,
            user_data_dir: temp_dir.path().to_str().unwrap().to_string(),
        };

        let mut shell = BrowserShell::new();
        shell.initialize(config).await.unwrap();

        let result = shell.shutdown().await;

        assert!(result.is_ok(), "Shutdown should succeed");
        assert!(shell.window_manager.is_none());
        assert!(shell.tab_manager.is_none());
    }

    #[tokio::test]
    async fn test_shutdown_without_initialization() {
        let mut shell = BrowserShell::new();

        let result = shell.shutdown().await;

        assert!(
            result.is_ok(),
            "Shutdown should succeed even without initialization"
        );
    }

    #[tokio::test]
    async fn test_new_window_without_initialization() {
        let mut shell = BrowserShell::new();

        let result = shell.new_window(None).await;

        assert!(
            result.is_err(),
            "Creating window without initialization should fail"
        );
        match result {
            Err(ComponentError::InvalidState(_)) => (),
            _ => panic!("Expected InvalidState error"),
        }
    }

    #[tokio::test]
    async fn test_new_tab_without_active_window() {
        let temp_dir = TempDir::new().unwrap();
        let config = ShellConfig {
            window_config: WindowConfig::default(),
            enable_devtools: true,
            enable_extensions: false,
            user_data_dir: temp_dir.path().to_str().unwrap().to_string(),
        };

        let mut shell = BrowserShell::new();
        shell.initialize(config).await.unwrap();

        let result = shell.new_tab(None).await;

        assert!(
            result.is_err(),
            "Creating tab without active window should fail"
        );
        match result {
            Err(ComponentError::InvalidState(_)) => (),
            _ => panic!("Expected InvalidState error"),
        }
    }

    #[tokio::test]
    async fn test_navigate_without_active_tab() {
        let temp_dir = TempDir::new().unwrap();
        let config = ShellConfig {
            window_config: WindowConfig::default(),
            enable_devtools: true,
            enable_extensions: false,
            user_data_dir: temp_dir.path().to_str().unwrap().to_string(),
        };

        let mut shell = BrowserShell::new();
        shell.initialize(config).await.unwrap();

        let result = shell.navigate("https://example.com".to_string()).await;

        assert!(result.is_err(), "Navigating without active tab should fail");
        match result {
            Err(ComponentError::InvalidState(_)) => (),
            _ => panic!("Expected InvalidState error"),
        }
    }

    #[tokio::test]
    async fn test_active_window_returns_none_initially() {
        let shell = BrowserShell::new();

        assert!(shell.active_window().is_none());
    }

    #[tokio::test]
    async fn test_active_tab_returns_none_initially() {
        let shell = BrowserShell::new();

        assert!(shell.active_tab().is_none());
    }
}
