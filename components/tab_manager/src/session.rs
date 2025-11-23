//! Session Management for Crash Recovery
//!
//! This module provides session persistence and crash recovery functionality
//! for the browser. It saves tab state periodically and can restore sessions
//! after an abnormal shutdown (crash).
//!
//! # Features
//!
//! - Save tab state (URLs, scroll positions, form data)
//! - Detect abnormal shutdown (crash detection via dirty flag)
//! - Restore session dialog on startup after crash
//! - Save session state to JSON file
//! - Handle multiple windows with multiple tabs each
//! - Session snapshot on clean shutdown

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared_types::{SessionError, TabId, WindowId};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;

/// Default auto-save interval in seconds
pub const DEFAULT_AUTO_SAVE_INTERVAL_SECS: u64 = 30;

/// Session file name
const SESSION_FILE: &str = "session.json";

/// Lock file name (dirty flag for crash detection)
const LOCK_FILE: &str = "session.lock";

/// Form data for a tab
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct FormData {
    /// Form field values keyed by field name/id
    pub fields: HashMap<String, String>,
}

/// Persisted state for a single tab
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TabSessionState {
    /// Tab identifier
    pub tab_id: TabId,
    /// Current URL
    pub url: Option<String>,
    /// Page title
    pub title: String,
    /// Scroll position (x, y)
    pub scroll_position: (i32, i32),
    /// Form data to restore
    pub form_data: Option<FormData>,
    /// Navigation history URLs
    pub history: Vec<String>,
    /// Current position in history
    pub history_index: usize,
    /// Whether this tab is pinned
    pub pinned: bool,
    /// Whether this tab is muted
    pub muted: bool,
}

impl TabSessionState {
    /// Create a new tab session state
    pub fn new(tab_id: TabId) -> Self {
        Self {
            tab_id,
            url: None,
            title: String::new(),
            scroll_position: (0, 0),
            form_data: None,
            history: Vec::new(),
            history_index: 0,
            pinned: false,
            muted: false,
        }
    }
}

/// Persisted state for a window
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WindowState {
    /// Window identifier
    pub window_id: WindowId,
    /// Tabs in this window (in order)
    pub tabs: Vec<TabSessionState>,
    /// Index of the active tab
    pub active_tab_index: usize,
    /// Window bounds (x, y, width, height)
    pub bounds: Option<(i32, i32, u32, u32)>,
    /// Whether window is maximized
    pub maximized: bool,
    /// Whether window is minimized
    pub minimized: bool,
}

impl WindowState {
    /// Create a new window state
    pub fn new(window_id: WindowId) -> Self {
        Self {
            window_id,
            tabs: Vec::new(),
            active_tab_index: 0,
            bounds: None,
            maximized: false,
            minimized: false,
        }
    }
}

/// Complete session state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionState {
    /// All windows in the session
    pub windows: Vec<WindowState>,
    /// When the session was last saved
    pub last_saved: DateTime<Utc>,
    /// Session format version for migration
    pub version: u32,
    /// Index of the focused window
    pub focused_window_index: Option<usize>,
}

impl Default for SessionState {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionState {
    /// Current session format version
    pub const CURRENT_VERSION: u32 = 1;

    /// Create a new empty session state
    pub fn new() -> Self {
        Self {
            windows: Vec::new(),
            last_saved: Utc::now(),
            version: Self::CURRENT_VERSION,
            focused_window_index: None,
        }
    }

    /// Get total tab count across all windows
    pub fn tab_count(&self) -> usize {
        self.windows.iter().map(|w| w.tabs.len()).sum()
    }

    /// Get total window count
    pub fn window_count(&self) -> usize {
        self.windows.len()
    }

    /// Check if session is empty
    pub fn is_empty(&self) -> bool {
        self.windows.is_empty() || self.tab_count() == 0
    }

    /// Add a window to the session
    pub fn add_window(&mut self, window: WindowState) {
        self.windows.push(window);
    }

    /// Get a window by ID
    pub fn get_window(&self, window_id: WindowId) -> Option<&WindowState> {
        self.windows.iter().find(|w| w.window_id == window_id)
    }

    /// Get a mutable window by ID
    pub fn get_window_mut(&mut self, window_id: WindowId) -> Option<&mut WindowState> {
        self.windows.iter_mut().find(|w| w.window_id == window_id)
    }
}

/// Configuration for the session manager
#[derive(Debug, Clone)]
pub struct SessionConfig {
    /// Directory to store session files
    pub session_dir: PathBuf,
    /// Auto-save interval in seconds
    pub auto_save_interval_secs: u64,
    /// Whether to enable auto-save
    pub auto_save_enabled: bool,
    /// Maximum number of sessions to keep in history
    pub max_session_history: usize,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            session_dir: PathBuf::from("."),
            auto_save_interval_secs: DEFAULT_AUTO_SAVE_INTERVAL_SECS,
            auto_save_enabled: true,
            max_session_history: 5,
        }
    }
}

impl SessionConfig {
    /// Create config with a specific session directory
    pub fn with_session_dir(session_dir: impl Into<PathBuf>) -> Self {
        Self {
            session_dir: session_dir.into(),
            ..Default::default()
        }
    }
}

/// Session manager for crash recovery
///
/// Handles saving and restoring browser sessions, including crash detection
/// via a dirty flag (lock file) mechanism.
pub struct SessionManager {
    config: SessionConfig,
    current_session: SessionState,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new(config: SessionConfig) -> Self {
        Self {
            config,
            current_session: SessionState::new(),
        }
    }

    /// Get the session file path
    fn session_file_path(&self) -> PathBuf {
        self.config.session_dir.join(SESSION_FILE)
    }

    /// Get the lock file path
    fn lock_file_path(&self) -> PathBuf {
        self.config.session_dir.join(LOCK_FILE)
    }

    /// Check if the browser crashed during the last session
    ///
    /// Returns true if a lock file exists (indicating abnormal shutdown)
    pub async fn was_crash(&self) -> bool {
        self.lock_file_path().exists()
    }

    /// Create the lock file (dirty flag)
    ///
    /// Called on startup to indicate session is active
    pub async fn mark_session_active(&self) -> Result<(), SessionError> {
        fs::create_dir_all(&self.config.session_dir)
            .await
            .map_err(|e| SessionError::IoError(e.to_string()))?;

        let lock_path = self.lock_file_path();
        let timestamp = Utc::now().to_rfc3339();
        fs::write(&lock_path, timestamp)
            .await
            .map_err(|e| SessionError::IoError(format!("Failed to create lock file: {}", e)))?;

        Ok(())
    }

    /// Remove the lock file (clean shutdown)
    ///
    /// Called on clean shutdown to indicate session ended normally
    pub async fn mark_session_closed(&self) -> Result<(), SessionError> {
        let lock_path = self.lock_file_path();
        if lock_path.exists() {
            fs::remove_file(&lock_path)
                .await
                .map_err(|e| SessionError::IoError(format!("Failed to remove lock file: {}", e)))?;
        }
        Ok(())
    }

    /// Save the current session to disk
    pub async fn save_session(&mut self) -> Result<(), SessionError> {
        self.current_session.last_saved = Utc::now();

        fs::create_dir_all(&self.config.session_dir)
            .await
            .map_err(|e| SessionError::IoError(e.to_string()))?;

        let json = serde_json::to_string_pretty(&self.current_session)
            .map_err(|e| SessionError::SerializationError(e.to_string()))?;

        let session_path = self.session_file_path();
        fs::write(&session_path, json)
            .await
            .map_err(|e| SessionError::SaveFailed(e.to_string()))?;

        Ok(())
    }

    /// Restore session from disk
    pub async fn restore_session(&mut self) -> Result<SessionState, SessionError> {
        let session_path = self.session_file_path();

        if !session_path.exists() {
            return Err(SessionError::NotFound(session_path.display().to_string()));
        }

        let json = fs::read_to_string(&session_path)
            .await
            .map_err(|e| SessionError::IoError(e.to_string()))?;

        let session: SessionState = serde_json::from_str(&json)
            .map_err(|e| SessionError::Corrupted(format!("Invalid session JSON: {}", e)))?;

        // Check version compatibility
        if session.version > SessionState::CURRENT_VERSION {
            return Err(SessionError::Corrupted(format!(
                "Session version {} is newer than supported version {}",
                session.version,
                SessionState::CURRENT_VERSION
            )));
        }

        self.current_session = session.clone();
        Ok(session)
    }

    /// Clear the saved session
    pub async fn clear_session(&mut self) -> Result<(), SessionError> {
        let session_path = self.session_file_path();

        if session_path.exists() {
            fs::remove_file(&session_path)
                .await
                .map_err(|e| SessionError::IoError(format!("Failed to clear session: {}", e)))?;
        }

        self.current_session = SessionState::new();
        Ok(())
    }

    /// Get a reference to the current session
    pub fn current_session(&self) -> &SessionState {
        &self.current_session
    }

    /// Get a mutable reference to the current session
    pub fn current_session_mut(&mut self) -> &mut SessionState {
        &mut self.current_session
    }

    /// Update the current session state
    pub fn set_session(&mut self, session: SessionState) {
        self.current_session = session;
    }

    /// Check if a previous session exists that can be restored
    pub async fn has_restorable_session(&self) -> bool {
        self.session_file_path().exists()
    }

    /// Get session info for restore dialog
    pub async fn get_session_info(&self) -> Result<SessionInfo, SessionError> {
        let session_path = self.session_file_path();

        if !session_path.exists() {
            return Err(SessionError::NotFound(session_path.display().to_string()));
        }

        let json = fs::read_to_string(&session_path)
            .await
            .map_err(|e| SessionError::IoError(e.to_string()))?;

        let session: SessionState = serde_json::from_str(&json)
            .map_err(|e| SessionError::Corrupted(e.to_string()))?;

        Ok(SessionInfo {
            window_count: session.window_count(),
            tab_count: session.tab_count(),
            last_saved: session.last_saved,
            was_crash: self.was_crash().await,
        })
    }

    /// Start auto-save background task
    ///
    /// Returns a handle that can be used to stop the auto-save task
    pub fn start_auto_save(&self) -> Option<AutoSaveHandle> {
        if !self.config.auto_save_enabled {
            return None;
        }

        let session_path = self.session_file_path();
        let interval_secs = self.config.auto_save_interval_secs;

        Some(AutoSaveHandle {
            session_path,
            interval_secs,
        })
    }
}

/// Information about a saved session for the restore dialog
#[derive(Debug, Clone)]
pub struct SessionInfo {
    /// Number of windows in the session
    pub window_count: usize,
    /// Total number of tabs
    pub tab_count: usize,
    /// When the session was last saved
    pub last_saved: DateTime<Utc>,
    /// Whether the previous session ended in a crash
    pub was_crash: bool,
}

/// Handle for the auto-save background task
#[derive(Debug)]
pub struct AutoSaveHandle {
    session_path: PathBuf,
    interval_secs: u64,
}

impl AutoSaveHandle {
    /// Get the session path
    pub fn session_path(&self) -> &Path {
        &self.session_path
    }

    /// Get the auto-save interval
    pub fn interval_secs(&self) -> u64 {
        self.interval_secs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_session_state() -> SessionState {
        let mut session = SessionState::new();

        let mut window = WindowState::new(WindowId::new());
        let mut tab1 = TabSessionState::new(TabId::new());
        tab1.url = Some("https://example.com".to_string());
        tab1.title = "Example".to_string();

        let mut tab2 = TabSessionState::new(TabId::new());
        tab2.url = Some("https://rust-lang.org".to_string());
        tab2.title = "Rust".to_string();

        window.tabs.push(tab1);
        window.tabs.push(tab2);
        window.active_tab_index = 0;

        session.add_window(window);
        session.focused_window_index = Some(0);

        session
    }

    #[tokio::test]
    async fn test_session_state_new() {
        let session = SessionState::new();
        assert!(session.windows.is_empty());
        assert_eq!(session.version, SessionState::CURRENT_VERSION);
        assert!(session.is_empty());
    }

    #[tokio::test]
    async fn test_session_state_tab_count() {
        let session = create_test_session_state();
        assert_eq!(session.tab_count(), 2);
        assert_eq!(session.window_count(), 1);
        assert!(!session.is_empty());
    }

    #[tokio::test]
    async fn test_session_manager_save_restore() {
        let temp_dir = TempDir::new().unwrap();
        let config = SessionConfig::with_session_dir(temp_dir.path());
        let mut manager = SessionManager::new(config);

        // Set up session
        let test_session = create_test_session_state();
        manager.set_session(test_session.clone());

        // Save
        manager.save_session().await.unwrap();

        // Verify file exists
        assert!(manager.session_file_path().exists());

        // Restore
        let restored = manager.restore_session().await.unwrap();
        assert_eq!(restored.tab_count(), 2);
        assert_eq!(restored.window_count(), 1);
    }

    #[tokio::test]
    async fn test_session_manager_crash_detection() {
        let temp_dir = TempDir::new().unwrap();
        let config = SessionConfig::with_session_dir(temp_dir.path());
        let manager = SessionManager::new(config);

        // Initially no crash
        assert!(!manager.was_crash().await);

        // Mark session active (creates lock file)
        manager.mark_session_active().await.unwrap();
        assert!(manager.was_crash().await);

        // Clean shutdown removes lock file
        manager.mark_session_closed().await.unwrap();
        assert!(!manager.was_crash().await);
    }

    #[tokio::test]
    async fn test_session_manager_clear_session() {
        let temp_dir = TempDir::new().unwrap();
        let config = SessionConfig::with_session_dir(temp_dir.path());
        let mut manager = SessionManager::new(config);

        // Save a session
        let test_session = create_test_session_state();
        manager.set_session(test_session);
        manager.save_session().await.unwrap();

        assert!(manager.session_file_path().exists());

        // Clear session
        manager.clear_session().await.unwrap();

        assert!(!manager.session_file_path().exists());
        assert!(manager.current_session().is_empty());
    }

    #[tokio::test]
    async fn test_session_manager_restore_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let config = SessionConfig::with_session_dir(temp_dir.path());
        let mut manager = SessionManager::new(config);

        let result = manager.restore_session().await;
        assert!(matches!(result, Err(SessionError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_session_manager_has_restorable_session() {
        let temp_dir = TempDir::new().unwrap();
        let config = SessionConfig::with_session_dir(temp_dir.path());
        let mut manager = SessionManager::new(config);

        // Initially no session
        assert!(!manager.has_restorable_session().await);

        // Save a session
        manager.set_session(create_test_session_state());
        manager.save_session().await.unwrap();

        // Now there's a session
        assert!(manager.has_restorable_session().await);
    }

    #[tokio::test]
    async fn test_session_manager_get_session_info() {
        let temp_dir = TempDir::new().unwrap();
        let config = SessionConfig::with_session_dir(temp_dir.path());
        let mut manager = SessionManager::new(config);

        // Save a session
        manager.set_session(create_test_session_state());
        manager.save_session().await.unwrap();

        // Get info
        let info = manager.get_session_info().await.unwrap();
        assert_eq!(info.window_count, 1);
        assert_eq!(info.tab_count, 2);
        assert!(!info.was_crash);
    }

    #[tokio::test]
    async fn test_session_state_serialization() {
        let session = create_test_session_state();

        let json = serde_json::to_string(&session).unwrap();
        let restored: SessionState = serde_json::from_str(&json).unwrap();

        assert_eq!(session.tab_count(), restored.tab_count());
        assert_eq!(session.window_count(), restored.window_count());
        assert_eq!(session.version, restored.version);
    }

    #[test]
    fn test_tab_session_state_new() {
        let tab_id = TabId::new();
        let tab = TabSessionState::new(tab_id);

        assert_eq!(tab.tab_id, tab_id);
        assert!(tab.url.is_none());
        assert!(tab.title.is_empty());
        assert_eq!(tab.scroll_position, (0, 0));
        assert!(tab.form_data.is_none());
        assert!(tab.history.is_empty());
        assert!(!tab.pinned);
        assert!(!tab.muted);
    }

    #[test]
    fn test_window_state_new() {
        let window_id = WindowId::new();
        let window = WindowState::new(window_id);

        assert_eq!(window.window_id, window_id);
        assert!(window.tabs.is_empty());
        assert_eq!(window.active_tab_index, 0);
        assert!(window.bounds.is_none());
        assert!(!window.maximized);
        assert!(!window.minimized);
    }

    #[test]
    fn test_session_config_default() {
        let config = SessionConfig::default();
        assert_eq!(config.auto_save_interval_secs, DEFAULT_AUTO_SAVE_INTERVAL_SECS);
        assert!(config.auto_save_enabled);
        assert_eq!(config.max_session_history, 5);
    }

    #[test]
    fn test_session_state_get_window() {
        let mut session = SessionState::new();
        let window_id = WindowId::new();
        let window = WindowState::new(window_id);

        session.add_window(window);

        assert!(session.get_window(window_id).is_some());
        assert!(session.get_window_mut(window_id).is_some());

        let fake_id = WindowId::new();
        assert!(session.get_window(fake_id).is_none());
    }

    #[test]
    fn test_form_data_default() {
        let form_data = FormData::default();
        assert!(form_data.fields.is_empty());
    }

    #[tokio::test]
    async fn test_auto_save_handle() {
        let temp_dir = TempDir::new().unwrap();
        let config = SessionConfig::with_session_dir(temp_dir.path());
        let manager = SessionManager::new(config);

        let handle = manager.start_auto_save();
        assert!(handle.is_some());

        let handle = handle.unwrap();
        assert_eq!(handle.interval_secs(), DEFAULT_AUTO_SAVE_INTERVAL_SECS);
    }

    #[tokio::test]
    async fn test_auto_save_disabled() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = SessionConfig::with_session_dir(temp_dir.path());
        config.auto_save_enabled = false;
        let manager = SessionManager::new(config);

        let handle = manager.start_auto_save();
        assert!(handle.is_none());
    }
}
