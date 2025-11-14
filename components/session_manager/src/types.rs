//! Session management data types

use serde::{Deserialize, Serialize};

/// Complete session state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionState {
    /// Windows in this session
    pub windows: Vec<WindowState>,
    /// Timestamp when session was saved (Unix timestamp)
    pub timestamp: i64,
}

/// Window state within a session
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WindowState {
    /// Window ID as string for serialization
    pub id: String,
    /// X coordinate (None if not set)
    pub x: Option<i32>,
    /// Y coordinate (None if not set)
    pub y: Option<i32>,
    /// Window width
    pub width: u32,
    /// Window height
    pub height: u32,
    /// Whether window is maximized
    pub maximized: bool,
    /// Tabs in this window
    pub tabs: Vec<TabState>,
    /// Index of active tab (None if no tabs)
    pub active_tab_index: Option<usize>,
}

/// Tab state within a window
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TabState {
    /// Tab ID as string for serialization
    pub id: String,
    /// Current URL
    pub url: String,
    /// Tab title
    pub title: String,
    /// Position in window
    pub position: usize,
}

/// Recently closed tab information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClosedTab {
    /// Tab ID
    pub id: String,
    /// Tab URL
    pub url: String,
    /// Tab title
    pub title: String,
    /// When tab was closed (Unix timestamp)
    pub closed_at: i64,
    /// Window ID this tab belonged to
    pub window_id: Option<String>,
    /// Position tab had in window
    pub position: Option<usize>,
}

impl SessionState {
    /// Create a new empty session state
    pub fn new(timestamp: i64) -> Self {
        Self {
            windows: Vec::new(),
            timestamp,
        }
    }
}

impl WindowState {
    /// Create a new window state
    pub fn new(id: String, width: u32, height: u32) -> Self {
        Self {
            id,
            x: None,
            y: None,
            width,
            height,
            maximized: false,
            tabs: Vec::new(),
            active_tab_index: None,
        }
    }
}

impl TabState {
    /// Create a new tab state
    pub fn new(id: String, url: String, title: String, position: usize) -> Self {
        Self {
            id,
            url,
            title,
            position,
        }
    }
}

impl ClosedTab {
    /// Create a new closed tab
    pub fn new(id: String, url: String, title: String, closed_at: i64) -> Self {
        Self {
            id,
            url,
            title,
            closed_at,
            window_id: None,
            position: None,
        }
    }
}
