//! Chrome-compatible Windows API (chrome.windows)
//!
//! Provides window query, creation, update, and removal functionality
//! compatible with Chrome's extension windows API.

use crate::api::ExtensionApi;
use crate::permissions::Permission;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Windows API error types
#[derive(Error, Debug)]
pub enum WindowsApiError {
    #[error("Window not found: {0}")]
    NotFound(u64),

    #[error("Permission denied")]
    PermissionDenied,

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("Operation failed: {0}")]
    OperationFailed(String),
}

/// Window state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WindowState {
    /// Normal window state
    Normal,
    /// Minimized
    Minimized,
    /// Maximized
    Maximized,
    /// Fullscreen
    Fullscreen,
    /// Locked fullscreen (cannot be escaped by user)
    LockedFullscreen,
}

impl Default for WindowState {
    fn default() -> Self {
        Self::Normal
    }
}

/// Window type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WindowType {
    /// Standard browser window
    Normal,
    /// Popup window
    Popup,
    /// Panel window (deprecated in Chrome)
    Panel,
    /// App window
    App,
    /// DevTools window
    Devtools,
}

impl Default for WindowType {
    fn default() -> Self {
        Self::Normal
    }
}

/// Window information returned by the API
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowInfo {
    /// Window ID
    pub id: u64,

    /// Whether this is the currently focused window
    pub focused: bool,

    /// Top position of the window
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top: Option<i32>,

    /// Left position of the window
    #[serde(skip_serializing_if = "Option::is_none")]
    pub left: Option<i32>,

    /// Width of the window
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,

    /// Height of the window
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,

    /// Tab IDs in the window
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tabs: Option<Vec<u64>>,

    /// Whether this is an incognito window
    pub incognito: bool,

    /// Window type
    #[serde(rename = "type")]
    pub window_type: WindowType,

    /// Window state
    pub state: WindowState,

    /// Whether always on top
    pub always_on_top: bool,

    /// Session ID for restored sessions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

/// Query options for getting windows
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetWindowInfo {
    /// Whether to populate the tabs array
    #[serde(skip_serializing_if = "Option::is_none")]
    pub populate: Option<bool>,

    /// Window types to include
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_types: Option<Vec<WindowType>>,
}

/// Options for creating a new window
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateWindowOptions {
    /// URL or list of URLs to open in the window
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<Vec<String>>,

    /// Tab ID to move to the new window
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tab_id: Option<u64>,

    /// Left position
    #[serde(skip_serializing_if = "Option::is_none")]
    pub left: Option<i32>,

    /// Top position
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top: Option<i32>,

    /// Width
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,

    /// Height
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,

    /// Whether to focus the window
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focused: Option<bool>,

    /// Whether to create an incognito window
    #[serde(skip_serializing_if = "Option::is_none")]
    pub incognito: Option<bool>,

    /// Window type
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub window_type: Option<WindowType>,

    /// Initial window state
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<WindowState>,

    /// Whether to show window decorations (setSelfAsOpener)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub set_self_as_opener: Option<bool>,
}

/// Options for updating a window
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateWindowOptions {
    /// Left position
    #[serde(skip_serializing_if = "Option::is_none")]
    pub left: Option<i32>,

    /// Top position
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top: Option<i32>,

    /// Width
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,

    /// Height
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,

    /// Whether to focus the window
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focused: Option<bool>,

    /// Whether the window is minimized (deprecated, use state)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub draw_attention: Option<bool>,

    /// Window state
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<WindowState>,
}

/// Chrome-compatible Windows API
///
/// Provides methods matching chrome.windows API for extension compatibility.
pub struct WindowsApi {
    /// Callback for getting all windows
    get_all_callback: Option<Box<dyn Fn(GetWindowInfo) -> Vec<WindowInfo> + Send + Sync>>,

    /// ID of the current window (for CURRENT_WINDOW constant)
    current_window_id: Option<u64>,

    /// ID of the last focused window
    last_focused_window_id: Option<u64>,
}

impl WindowsApi {
    /// Special window ID representing the current window
    pub const WINDOW_ID_CURRENT: i64 = -2;

    /// Special window ID representing no window
    pub const WINDOW_ID_NONE: i64 = -1;

    /// Create a new WindowsApi
    pub fn new() -> Self {
        Self {
            get_all_callback: None,
            current_window_id: None,
            last_focused_window_id: None,
        }
    }

    /// Set the callback for getting all windows
    pub fn set_get_all_callback(
        &mut self,
        callback: Box<dyn Fn(GetWindowInfo) -> Vec<WindowInfo> + Send + Sync>,
    ) {
        self.get_all_callback = Some(callback);
    }

    /// Set the current window ID
    pub fn set_current_window(&mut self, window_id: u64) {
        self.current_window_id = Some(window_id);
    }

    /// Set the last focused window ID
    pub fn set_last_focused_window(&mut self, window_id: u64) {
        self.last_focused_window_id = Some(window_id);
    }

    /// Get a window by ID
    ///
    /// # Arguments
    ///
    /// * `window_id` - ID of the window to get
    /// * `get_info` - Options for the query
    ///
    /// # Returns
    ///
    /// Window information if found
    pub fn get(
        &self,
        window_id: i64,
        get_info: Option<GetWindowInfo>,
    ) -> Result<WindowInfo, WindowsApiError> {
        let actual_id = self.resolve_window_id(window_id)?;
        let info = get_info.unwrap_or_default();

        let windows = self.get_all(Some(info))?;
        windows
            .into_iter()
            .find(|w| w.id == actual_id)
            .ok_or(WindowsApiError::NotFound(actual_id))
    }

    /// Get the currently focused window
    ///
    /// # Arguments
    ///
    /// * `get_info` - Options for the query
    ///
    /// # Returns
    ///
    /// Window information if there is a focused window
    pub fn get_current(&self, get_info: Option<GetWindowInfo>) -> Result<WindowInfo, WindowsApiError> {
        if let Some(id) = self.current_window_id {
            self.get(id as i64, get_info)
        } else {
            Err(WindowsApiError::OperationFailed(
                "No current window".to_string(),
            ))
        }
    }

    /// Get the last focused window
    ///
    /// # Arguments
    ///
    /// * `get_info` - Options for the query
    ///
    /// # Returns
    ///
    /// Window information
    pub fn get_last_focused(
        &self,
        get_info: Option<GetWindowInfo>,
    ) -> Result<WindowInfo, WindowsApiError> {
        if let Some(id) = self.last_focused_window_id {
            self.get(id as i64, get_info)
        } else {
            Err(WindowsApiError::OperationFailed(
                "No last focused window".to_string(),
            ))
        }
    }

    /// Get all windows
    ///
    /// # Arguments
    ///
    /// * `get_info` - Options for the query
    ///
    /// # Returns
    ///
    /// Vector of all windows
    pub fn get_all(&self, get_info: Option<GetWindowInfo>) -> Result<Vec<WindowInfo>, WindowsApiError> {
        if let Some(ref callback) = self.get_all_callback {
            Ok(callback(get_info.unwrap_or_default()))
        } else {
            Ok(Vec::new())
        }
    }

    /// Create a new window
    ///
    /// # Arguments
    ///
    /// * `options` - Creation options
    ///
    /// # Returns
    ///
    /// Created window information
    pub fn create(&self, _options: Option<CreateWindowOptions>) -> Result<WindowInfo, WindowsApiError> {
        Err(WindowsApiError::OperationFailed(
            "Window creation requires integration with WindowManager".to_string(),
        ))
    }

    /// Update an existing window
    ///
    /// # Arguments
    ///
    /// * `window_id` - ID of the window to update
    /// * `options` - Update options
    ///
    /// # Returns
    ///
    /// Updated window information
    pub fn update(
        &self,
        _window_id: i64,
        _options: UpdateWindowOptions,
    ) -> Result<WindowInfo, WindowsApiError> {
        Err(WindowsApiError::OperationFailed(
            "Window update requires integration with WindowManager".to_string(),
        ))
    }

    /// Remove (close) a window
    ///
    /// # Arguments
    ///
    /// * `window_id` - ID of the window to remove
    ///
    /// # Returns
    ///
    /// Success or error
    pub fn remove(&self, _window_id: i64) -> Result<(), WindowsApiError> {
        Err(WindowsApiError::OperationFailed(
            "Window removal requires integration with WindowManager".to_string(),
        ))
    }

    /// Resolve special window ID values
    fn resolve_window_id(&self, window_id: i64) -> Result<u64, WindowsApiError> {
        if window_id == Self::WINDOW_ID_CURRENT {
            self.current_window_id
                .ok_or_else(|| WindowsApiError::OperationFailed("No current window".to_string()))
        } else if window_id == Self::WINDOW_ID_NONE || window_id < 0 {
            Err(WindowsApiError::InvalidArgument(
                "Invalid window ID".to_string(),
            ))
        } else {
            Ok(window_id as u64)
        }
    }
}

impl Default for WindowsApi {
    fn default() -> Self {
        Self::new()
    }
}

impl ExtensionApi for WindowsApi {
    fn namespace(&self) -> &str {
        "windows"
    }

    fn required_permission(&self) -> Permission {
        // Windows API has implicit permission with tabs
        Permission::Tabs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_windows_api_creation() {
        let api = WindowsApi::new();
        assert_eq!(api.namespace(), "windows");
    }

    #[test]
    fn test_window_state_serialization() {
        assert_eq!(
            serde_json::to_string(&WindowState::Normal).unwrap(),
            "\"normal\""
        );
        assert_eq!(
            serde_json::to_string(&WindowState::Maximized).unwrap(),
            "\"maximized\""
        );
    }

    #[test]
    fn test_window_type_serialization() {
        assert_eq!(
            serde_json::to_string(&WindowType::Normal).unwrap(),
            "\"normal\""
        );
        assert_eq!(
            serde_json::to_string(&WindowType::Popup).unwrap(),
            "\"popup\""
        );
    }

    #[test]
    fn test_get_all_with_callback() {
        let mut api = WindowsApi::new();
        api.set_get_all_callback(Box::new(|_info| {
            vec![WindowInfo {
                id: 1,
                focused: true,
                top: Some(0),
                left: Some(0),
                width: Some(800),
                height: Some(600),
                tabs: None,
                incognito: false,
                window_type: WindowType::Normal,
                state: WindowState::Normal,
                always_on_top: false,
                session_id: None,
            }]
        }));

        let result = api.get_all(None);
        assert!(result.is_ok());
        let windows = result.unwrap();
        assert_eq!(windows.len(), 1);
        assert_eq!(windows[0].id, 1);
    }

    #[test]
    fn test_get_current_window() {
        let mut api = WindowsApi::new();
        api.set_current_window(42);
        api.set_get_all_callback(Box::new(|_info| {
            vec![WindowInfo {
                id: 42,
                focused: true,
                top: None,
                left: None,
                width: None,
                height: None,
                tabs: None,
                incognito: false,
                window_type: WindowType::Normal,
                state: WindowState::Normal,
                always_on_top: false,
                session_id: None,
            }]
        }));

        let result = api.get_current(None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, 42);
    }

    #[test]
    fn test_resolve_window_id_current() {
        let mut api = WindowsApi::new();
        api.set_current_window(42);

        let result = api.resolve_window_id(WindowsApi::WINDOW_ID_CURRENT);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_resolve_window_id_invalid() {
        let api = WindowsApi::new();

        let result = api.resolve_window_id(WindowsApi::WINDOW_ID_NONE);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_window_options() {
        let options = CreateWindowOptions {
            url: Some(vec!["https://example.com".to_string()]),
            width: Some(800),
            height: Some(600),
            ..Default::default()
        };
        let json = serde_json::to_string(&options).unwrap();
        assert!(json.contains("url"));
        assert!(json.contains("width"));
    }
}
