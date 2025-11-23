//! Multi-window tab drag and drop support
//!
//! This module provides cross-window tab drag and drop functionality,
//! enabling tabs to be moved between browser windows seamlessly.
//!
//! # Architecture
//!
//! The tab drag system consists of:
//! - [`TabDragManager`]: Central coordinator for all drag operations
//! - [`DragSession`]: Tracks the state of an active drag operation
//! - [`WindowDropTarget`]: Represents a window that can receive dragged tabs
//! - [`TabTransferData`]: Serializable tab state for cross-window transfer
//!
//! # Example
//!
//! ```rust,ignore
//! use window_manager::tab_drag::{TabDragManager, TabDragState};
//! use shared_types::{TabId, WindowId};
//!
//! let mut manager = TabDragManager::new();
//!
//! // Start dragging a tab
//! manager.start_drag(tab_id, source_window_id, cursor_position);
//!
//! // Update drag position
//! manager.update_drag_position(new_position);
//!
//! // Check if over a valid drop target
//! if let Some(target) = manager.get_hover_target() {
//!     // Complete the drop
//!     let transfer_data = manager.complete_drop(target.window_id).unwrap();
//! }
//! ```

use serde::{Deserialize, Serialize};
use shared_types::{TabId, WindowId};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Current state of the tab drag operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TabDragState {
    /// No drag operation in progress
    Idle,
    /// Tab is being dragged
    Dragging,
    /// Dragging tab is hovering over a potential drop target
    Hovering,
    /// Tab is being dropped onto a target
    Dropping,
}

impl Default for TabDragState {
    fn default() -> Self {
        Self::Idle
    }
}

/// 2D position for cursor/drag tracking
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Position {
    /// X coordinate in screen pixels
    pub x: f64,
    /// Y coordinate in screen pixels
    pub y: f64,
}

impl Position {
    /// Create a new position
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Calculate distance to another position
    pub fn distance_to(&self, other: &Position) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

impl Default for Position {
    fn default() -> Self {
        Self::new(0.0, 0.0)
    }
}

/// Rectangular region for hit testing
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rectangle {
    /// Left edge X coordinate
    pub x: f64,
    /// Top edge Y coordinate
    pub y: f64,
    /// Width of the rectangle
    pub width: f64,
    /// Height of the rectangle
    pub height: f64,
}

impl Rectangle {
    /// Create a new rectangle
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Check if a position is within this rectangle
    pub fn contains(&self, pos: &Position) -> bool {
        pos.x >= self.x
            && pos.x <= self.x + self.width
            && pos.y >= self.y
            && pos.y <= self.y + self.height
    }

    /// Get the center position of this rectangle
    pub fn center(&self) -> Position {
        Position::new(self.x + self.width / 2.0, self.y + self.height / 2.0)
    }
}

/// History entry for tab transfer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// URL of the history entry
    pub url: String,
    /// Title of the page
    pub title: String,
}

impl HistoryEntry {
    /// Create a new history entry
    pub fn new(url: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            title: title.into(),
        }
    }
}

/// Complete tab state data for cross-window transfer
///
/// This structure contains all the information needed to reconstruct
/// a tab in a different window, including navigation history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabTransferData {
    /// Original tab ID
    pub tab_id: TabId,
    /// Current URL of the tab
    pub url: Option<String>,
    /// Current page title
    pub title: String,
    /// Navigation history entries
    pub history: Vec<HistoryEntry>,
    /// Current position in history (0-indexed)
    pub history_index: Option<usize>,
    /// Whether the tab is in private/incognito mode
    pub is_private: bool,
    /// Scroll position (x, y)
    pub scroll_position: Option<(i32, i32)>,
    /// Zoom level (1.0 = 100%)
    pub zoom_level: f64,
    /// Source window the tab is being dragged from
    pub source_window_id: WindowId,
    /// Timestamp of when the transfer data was created
    pub created_at: u64,
}

impl TabTransferData {
    /// Create new transfer data for a tab
    pub fn new(tab_id: TabId, source_window_id: WindowId) -> Self {
        Self {
            tab_id,
            url: None,
            title: String::new(),
            history: Vec::new(),
            history_index: None,
            is_private: false,
            scroll_position: None,
            zoom_level: 1.0,
            source_window_id,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        }
    }

    /// Set the current URL
    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    /// Set the title
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Add a history entry
    pub fn with_history_entry(mut self, entry: HistoryEntry) -> Self {
        self.history.push(entry);
        self
    }

    /// Set the history entries
    pub fn with_history(mut self, history: Vec<HistoryEntry>) -> Self {
        self.history = history;
        self
    }

    /// Set the current history index
    pub fn with_history_index(mut self, index: usize) -> Self {
        self.history_index = Some(index);
        self
    }

    /// Set private mode flag
    pub fn with_private(mut self, is_private: bool) -> Self {
        self.is_private = is_private;
        self
    }

    /// Set scroll position
    pub fn with_scroll_position(mut self, x: i32, y: i32) -> Self {
        self.scroll_position = Some((x, y));
        self
    }

    /// Set zoom level
    pub fn with_zoom_level(mut self, zoom: f64) -> Self {
        self.zoom_level = zoom;
        self
    }

    /// Serialize to JSON for IPC transfer
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserialize from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Check if this transfer data can navigate back
    pub fn can_go_back(&self) -> bool {
        self.history_index.map(|i| i > 0).unwrap_or(false)
    }

    /// Check if this transfer data can navigate forward
    pub fn can_go_forward(&self) -> bool {
        match self.history_index {
            Some(index) => index < self.history.len().saturating_sub(1),
            None => false,
        }
    }
}

/// Visual feedback information during drag operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DragFeedback {
    /// Current preview position (for ghost tab)
    pub preview_position: Position,
    /// Whether the current position is a valid drop target
    pub is_valid_drop: bool,
    /// Suggested drop index in target tab bar
    pub suggested_drop_index: Option<usize>,
    /// Target window ID if hovering over one
    pub target_window_id: Option<WindowId>,
    /// Visual indicator type
    pub indicator: DropIndicator,
}

impl DragFeedback {
    /// Create new feedback for an invalid drop position
    pub fn invalid(position: Position) -> Self {
        Self {
            preview_position: position,
            is_valid_drop: false,
            suggested_drop_index: None,
            target_window_id: None,
            indicator: DropIndicator::None,
        }
    }

    /// Create new feedback for a valid drop target
    pub fn valid(
        position: Position,
        target_window_id: WindowId,
        drop_index: usize,
    ) -> Self {
        Self {
            preview_position: position,
            is_valid_drop: true,
            suggested_drop_index: Some(drop_index),
            target_window_id: Some(target_window_id),
            indicator: DropIndicator::InsertMarker,
        }
    }
}

/// Type of visual indicator to display during drag
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DropIndicator {
    /// No indicator (invalid drop zone)
    None,
    /// Insert marker between tabs
    InsertMarker,
    /// Highlight entire tab bar
    TabBarHighlight,
    /// Create new window indicator
    NewWindow,
}

/// Represents a window that can accept dropped tabs
#[derive(Debug, Clone)]
pub struct WindowDropTarget {
    /// Window identifier
    pub window_id: WindowId,
    /// Window bounds for hit testing
    pub bounds: Rectangle,
    /// Tab bar region within the window
    pub tab_bar_bounds: Rectangle,
    /// Number of tabs currently in this window
    pub tab_count: usize,
    /// Whether this window can accept more tabs
    pub can_accept_tabs: bool,
}

impl WindowDropTarget {
    /// Create a new drop target for a window
    pub fn new(window_id: WindowId, bounds: Rectangle, tab_bar_bounds: Rectangle) -> Self {
        Self {
            window_id,
            bounds,
            tab_bar_bounds,
            tab_count: 0,
            can_accept_tabs: true,
        }
    }

    /// Set the current tab count
    pub fn with_tab_count(mut self, count: usize) -> Self {
        self.tab_count = count;
        self
    }

    /// Check if a position is within this window
    pub fn contains(&self, pos: &Position) -> bool {
        self.bounds.contains(pos)
    }

    /// Check if a position is within the tab bar
    pub fn tab_bar_contains(&self, pos: &Position) -> bool {
        self.tab_bar_bounds.contains(pos)
    }

    /// Calculate the drop index based on position within tab bar
    pub fn calculate_drop_index(&self, pos: &Position) -> usize {
        if !self.tab_bar_contains(pos) || self.tab_count == 0 {
            return 0;
        }

        // Calculate approximate tab width
        let tab_width = self.tab_bar_bounds.width / self.tab_count.max(1) as f64;
        let relative_x = pos.x - self.tab_bar_bounds.x;
        let index = (relative_x / tab_width).floor() as usize;

        index.min(self.tab_count)
    }
}

/// Active drag session tracking all drag state
#[derive(Debug, Clone)]
pub struct DragSession {
    /// Tab being dragged
    pub tab_id: TabId,
    /// Window the tab is being dragged from
    pub source_window_id: WindowId,
    /// Initial drag start position
    pub start_position: Position,
    /// Current cursor position
    pub current_position: Position,
    /// Current drag state
    pub state: TabDragState,
    /// Complete tab data for transfer
    pub transfer_data: TabTransferData,
    /// When the drag started
    pub started_at: Instant,
    /// Currently hovered window (if any)
    pub hover_window_id: Option<WindowId>,
    /// Time spent hovering over current window
    pub hover_duration: Duration,
    /// Offset from cursor to tab preview origin
    pub cursor_offset: Position,
}

impl DragSession {
    /// Create a new drag session
    pub fn new(
        tab_id: TabId,
        source_window_id: WindowId,
        start_position: Position,
        transfer_data: TabTransferData,
    ) -> Self {
        Self {
            tab_id,
            source_window_id,
            start_position,
            current_position: start_position,
            state: TabDragState::Dragging,
            transfer_data,
            started_at: Instant::now(),
            hover_window_id: None,
            hover_duration: Duration::ZERO,
            cursor_offset: Position::default(),
        }
    }

    /// Update the current position
    pub fn update_position(&mut self, position: Position) {
        self.current_position = position;
    }

    /// Set the hover state
    pub fn set_hover(&mut self, window_id: Option<WindowId>) {
        if self.hover_window_id != window_id {
            self.hover_window_id = window_id;
            self.hover_duration = Duration::ZERO;
            self.state = if window_id.is_some() {
                TabDragState::Hovering
            } else {
                TabDragState::Dragging
            };
        }
    }

    /// Add hover duration
    pub fn add_hover_duration(&mut self, duration: Duration) {
        self.hover_duration += duration;
    }

    /// Get drag distance from start
    pub fn drag_distance(&self) -> f64 {
        self.start_position.distance_to(&self.current_position)
    }

    /// Get total drag duration
    pub fn drag_duration(&self) -> Duration {
        self.started_at.elapsed()
    }

    /// Check if drag has moved enough to be considered intentional
    pub fn is_significant_drag(&self) -> bool {
        self.drag_distance() > 10.0
    }
}

/// Messages for cross-window IPC during drag operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrossWindowMessage {
    /// Notify other windows that a drag has started
    DragStarted {
        /// Tab being dragged
        tab_id: TabId,
        /// Source window
        source_window_id: WindowId,
        /// Current cursor position
        position: Position,
    },
    /// Update drag position (broadcast to all windows)
    DragMoved {
        /// New cursor position
        position: Position,
    },
    /// Drag entered a window's bounds
    DragEntered {
        /// Window being entered
        window_id: WindowId,
        /// Entry position
        position: Position,
    },
    /// Drag left a window's bounds
    DragLeft {
        /// Window being left
        window_id: WindowId,
    },
    /// Request to accept a dropped tab
    DropRequested {
        /// Target window
        target_window_id: WindowId,
        /// Tab transfer data (serialized)
        transfer_data_json: String,
        /// Drop index in tab bar
        drop_index: usize,
    },
    /// Confirm drop was accepted
    DropAccepted {
        /// Target window that accepted
        target_window_id: WindowId,
        /// New tab ID assigned in target window
        new_tab_id: TabId,
    },
    /// Drop was rejected
    DropRejected {
        /// Target window that rejected
        target_window_id: WindowId,
        /// Reason for rejection
        reason: String,
    },
    /// Drag was cancelled
    DragCancelled {
        /// Original tab ID
        tab_id: TabId,
    },
}

impl CrossWindowMessage {
    /// Serialize to JSON for IPC
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserialize from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

/// Error types for tab drag operations
#[derive(Debug, Clone, thiserror::Error)]
pub enum TabDragError {
    /// No drag operation is currently in progress
    #[error("No drag in progress")]
    NoDragInProgress,

    /// A drag operation is already in progress
    #[error("Drag already in progress")]
    DragAlreadyInProgress,

    /// The specified window was not found
    #[error("Window not found: {0:?}")]
    WindowNotFound(WindowId),

    /// The drop target is invalid
    #[error("Invalid drop target: {0}")]
    InvalidDropTarget(String),

    /// The drop was rejected by the target window
    #[error("Drop rejected: {0}")]
    DropRejected(String),

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// The tab cannot be dragged
    #[error("Tab cannot be dragged: {0}")]
    TabNotDraggable(String),
}

/// Central coordinator for cross-window tab drag operations
#[derive(Debug)]
pub struct TabDragManager {
    /// Currently active drag session (if any)
    active_session: Option<DragSession>,
    /// Registered drop targets (windows)
    drop_targets: HashMap<WindowId, WindowDropTarget>,
    /// Pending IPC messages to send
    outgoing_messages: Vec<CrossWindowMessage>,
    /// Minimum drag distance before drag is activated
    min_drag_distance: f64,
    /// Whether dragging to create a new window is enabled
    allow_new_window_drop: bool,
}

impl TabDragManager {
    /// Create a new TabDragManager
    pub fn new() -> Self {
        Self {
            active_session: None,
            drop_targets: HashMap::new(),
            outgoing_messages: Vec::new(),
            min_drag_distance: 10.0,
            allow_new_window_drop: true,
        }
    }

    /// Set the minimum drag distance before drag activates
    pub fn set_min_drag_distance(&mut self, distance: f64) {
        self.min_drag_distance = distance;
    }

    /// Set whether dropping to create new window is allowed
    pub fn set_allow_new_window_drop(&mut self, allow: bool) {
        self.allow_new_window_drop = allow;
    }

    /// Check if a drag is currently in progress
    pub fn is_dragging(&self) -> bool {
        self.active_session.is_some()
    }

    /// Get the current drag state
    pub fn get_state(&self) -> TabDragState {
        self.active_session
            .as_ref()
            .map(|s| s.state)
            .unwrap_or(TabDragState::Idle)
    }

    /// Get the current drag session (if any)
    pub fn get_session(&self) -> Option<&DragSession> {
        self.active_session.as_ref()
    }

    /// Register a window as a potential drop target
    pub fn register_drop_target(&mut self, target: WindowDropTarget) {
        self.drop_targets.insert(target.window_id, target);
    }

    /// Unregister a window drop target
    pub fn unregister_drop_target(&mut self, window_id: WindowId) {
        self.drop_targets.remove(&window_id);
    }

    /// Update drop target information
    pub fn update_drop_target(&mut self, window_id: WindowId, bounds: Rectangle, tab_bar_bounds: Rectangle, tab_count: usize) {
        if let Some(target) = self.drop_targets.get_mut(&window_id) {
            target.bounds = bounds;
            target.tab_bar_bounds = tab_bar_bounds;
            target.tab_count = tab_count;
        }
    }

    /// Start a new drag operation
    pub fn start_drag(
        &mut self,
        tab_id: TabId,
        source_window_id: WindowId,
        position: Position,
        transfer_data: TabTransferData,
    ) -> Result<(), TabDragError> {
        if self.active_session.is_some() {
            return Err(TabDragError::DragAlreadyInProgress);
        }

        let session = DragSession::new(tab_id, source_window_id, position, transfer_data);
        self.active_session = Some(session);

        // Queue IPC message
        self.outgoing_messages.push(CrossWindowMessage::DragStarted {
            tab_id,
            source_window_id,
            position,
        });

        Ok(())
    }

    /// Update the drag position
    pub fn update_drag_position(&mut self, position: Position) -> Result<DragFeedback, TabDragError> {
        // First check if session exists and get needed data
        if self.active_session.is_none() {
            return Err(TabDragError::NoDragInProgress);
        }

        // Get session info needed for feedback generation
        let (previous_hover, source_window_id) = {
            let session = self.active_session.as_mut().unwrap();
            session.update_position(position);
            (session.hover_window_id, session.source_window_id)
        };

        // Queue position update message
        self.outgoing_messages
            .push(CrossWindowMessage::DragMoved { position });

        // Find hover target - this borrows drop_targets immutably
        let hover_info = self.drop_targets.values()
            .find(|t| t.contains(&position))
            .map(|t| (
                t.window_id,
                t.can_accept_tabs,
                t.tab_bar_contains(&position),
                t.calculate_drop_index(&position),
            ));

        let new_hover = hover_info.map(|(id, _, _, _)| id);

        // Queue enter/leave messages
        if previous_hover != new_hover {
            if let Some(prev_id) = previous_hover {
                self.outgoing_messages
                    .push(CrossWindowMessage::DragLeft { window_id: prev_id });
            }
            if let Some(new_id) = new_hover {
                self.outgoing_messages
                    .push(CrossWindowMessage::DragEntered {
                        window_id: new_id,
                        position,
                    });
            }
        }

        // Update session hover state
        if let Some(session) = self.active_session.as_mut() {
            session.set_hover(new_hover);
        }

        // Generate feedback based on hover info
        let feedback = match hover_info {
            Some((window_id, can_accept, _in_tab_bar, drop_index)) if can_accept && window_id != source_window_id => {
                DragFeedback::valid(position, window_id, drop_index)
            }
            Some((window_id, _, in_tab_bar, drop_index)) if window_id == source_window_id && in_tab_bar => {
                // Reordering within same window
                DragFeedback::valid(position, window_id, drop_index)
            }
            _ if self.allow_new_window_drop => {
                // Could create new window
                DragFeedback {
                    preview_position: position,
                    is_valid_drop: true,
                    suggested_drop_index: Some(0),
                    target_window_id: None,
                    indicator: DropIndicator::NewWindow,
                }
            }
            _ => DragFeedback::invalid(position),
        };

        Ok(feedback)
    }

    /// Complete the drop operation
    pub fn complete_drop(&mut self, target_window_id: Option<WindowId>) -> Result<(TabTransferData, Option<usize>), TabDragError> {
        let session = self.active_session.take().ok_or(TabDragError::NoDragInProgress)?;

        // Get drop index if dropping on existing window
        let drop_index = match target_window_id {
            Some(window_id) => {
                let target = self.drop_targets.get(&window_id)
                    .ok_or(TabDragError::WindowNotFound(window_id))?;

                if !target.can_accept_tabs {
                    self.active_session = Some(session);
                    return Err(TabDragError::InvalidDropTarget(
                        "Window cannot accept tabs".to_string(),
                    ));
                }

                Some(target.calculate_drop_index(&session.current_position))
            }
            None if self.allow_new_window_drop => None,
            None => {
                self.active_session = Some(session);
                return Err(TabDragError::InvalidDropTarget(
                    "No target window and new window creation disabled".to_string(),
                ));
            }
        };

        // Queue drop request message
        if let Some(window_id) = target_window_id {
            if let Ok(json) = session.transfer_data.to_json() {
                self.outgoing_messages.push(CrossWindowMessage::DropRequested {
                    target_window_id: window_id,
                    transfer_data_json: json,
                    drop_index: drop_index.unwrap_or(0),
                });
            }
        }

        Ok((session.transfer_data, drop_index))
    }

    /// Cancel the current drag operation
    pub fn cancel_drag(&mut self) -> Result<TabId, TabDragError> {
        let session = self.active_session.take().ok_or(TabDragError::NoDragInProgress)?;

        self.outgoing_messages
            .push(CrossWindowMessage::DragCancelled {
                tab_id: session.tab_id,
            });

        Ok(session.tab_id)
    }

    /// Get and clear pending outgoing messages
    pub fn take_outgoing_messages(&mut self) -> Vec<CrossWindowMessage> {
        std::mem::take(&mut self.outgoing_messages)
    }

    /// Handle an incoming cross-window message
    pub fn handle_message(&mut self, message: CrossWindowMessage) -> Option<CrossWindowMessage> {
        match message {
            CrossWindowMessage::DragEntered { window_id, position } => {
                // Update session if we have one
                if let Some(session) = &mut self.active_session {
                    session.set_hover(Some(window_id));
                    session.update_position(position);
                }
                None
            }
            CrossWindowMessage::DragLeft { window_id } => {
                if let Some(session) = &mut self.active_session {
                    if session.hover_window_id == Some(window_id) {
                        session.set_hover(None);
                    }
                }
                None
            }
            CrossWindowMessage::DropRequested {
                target_window_id,
                transfer_data_json: _,
                drop_index: _,
            } => {
                // Verify we can accept this drop
                if let Some(target) = self.drop_targets.get(&target_window_id) {
                    if target.can_accept_tabs {
                        // Accept the drop
                        Some(CrossWindowMessage::DropAccepted {
                            target_window_id,
                            new_tab_id: TabId::new(),
                        })
                    } else {
                        Some(CrossWindowMessage::DropRejected {
                            target_window_id,
                            reason: "Window cannot accept more tabs".to_string(),
                        })
                    }
                } else {
                    Some(CrossWindowMessage::DropRejected {
                        target_window_id,
                        reason: "Window not found".to_string(),
                    })
                }
            }
            _ => None,
        }
    }

    /// Find the drop target at the given position
    #[allow(dead_code)]
    fn find_drop_target(&self, position: &Position) -> Option<&WindowDropTarget> {
        self.drop_targets.values().find(|t| t.contains(position))
    }

    /// Get the currently hovered drop target
    pub fn get_hover_target(&self) -> Option<&WindowDropTarget> {
        let session = self.active_session.as_ref()?;
        let window_id = session.hover_window_id?;
        self.drop_targets.get(&window_id)
    }

    /// Get the number of registered drop targets
    pub fn drop_target_count(&self) -> usize {
        self.drop_targets.len()
    }

    /// Check if a specific window is registered as a drop target
    pub fn has_drop_target(&self, window_id: WindowId) -> bool {
        self.drop_targets.contains_key(&window_id)
    }
}

impl Default for TabDragManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_transfer_data(tab_id: TabId, source_window_id: WindowId) -> TabTransferData {
        TabTransferData::new(tab_id, source_window_id)
            .with_url("https://example.com")
            .with_title("Example Page")
    }

    fn create_test_drop_target(window_id: WindowId) -> WindowDropTarget {
        let bounds = Rectangle::new(0.0, 0.0, 800.0, 600.0);
        let tab_bar_bounds = Rectangle::new(0.0, 0.0, 800.0, 40.0);
        WindowDropTarget::new(window_id, bounds, tab_bar_bounds).with_tab_count(5)
    }

    #[test]
    fn test_tab_drag_state_default() {
        assert_eq!(TabDragState::default(), TabDragState::Idle);
    }

    #[test]
    fn test_position_distance() {
        let p1 = Position::new(0.0, 0.0);
        let p2 = Position::new(3.0, 4.0);
        assert!((p1.distance_to(&p2) - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_rectangle_contains() {
        let rect = Rectangle::new(10.0, 10.0, 100.0, 50.0);

        assert!(rect.contains(&Position::new(50.0, 30.0)));
        assert!(rect.contains(&Position::new(10.0, 10.0)));
        assert!(rect.contains(&Position::new(110.0, 60.0)));
        assert!(!rect.contains(&Position::new(5.0, 30.0)));
        assert!(!rect.contains(&Position::new(50.0, 5.0)));
    }

    #[test]
    fn test_rectangle_center() {
        let rect = Rectangle::new(0.0, 0.0, 100.0, 50.0);
        let center = rect.center();
        assert_eq!(center.x, 50.0);
        assert_eq!(center.y, 25.0);
    }

    #[test]
    fn test_tab_transfer_data_builder() {
        let tab_id = TabId::new();
        let window_id = WindowId::new();

        let data = TabTransferData::new(tab_id, window_id)
            .with_url("https://example.com")
            .with_title("Test Page")
            .with_private(true)
            .with_scroll_position(100, 200)
            .with_zoom_level(1.5);

        assert_eq!(data.tab_id, tab_id);
        assert_eq!(data.url, Some("https://example.com".to_string()));
        assert_eq!(data.title, "Test Page");
        assert!(data.is_private);
        assert_eq!(data.scroll_position, Some((100, 200)));
        assert_eq!(data.zoom_level, 1.5);
    }

    #[test]
    fn test_tab_transfer_data_history_navigation() {
        let tab_id = TabId::new();
        let window_id = WindowId::new();

        let data = TabTransferData::new(tab_id, window_id)
            .with_history(vec![
                HistoryEntry::new("https://page1.com", "Page 1"),
                HistoryEntry::new("https://page2.com", "Page 2"),
                HistoryEntry::new("https://page3.com", "Page 3"),
            ])
            .with_history_index(1);

        assert!(data.can_go_back());
        assert!(data.can_go_forward());
    }

    #[test]
    fn test_tab_transfer_data_history_at_start() {
        let tab_id = TabId::new();
        let window_id = WindowId::new();

        let data = TabTransferData::new(tab_id, window_id)
            .with_history(vec![
                HistoryEntry::new("https://page1.com", "Page 1"),
                HistoryEntry::new("https://page2.com", "Page 2"),
            ])
            .with_history_index(0);

        assert!(!data.can_go_back());
        assert!(data.can_go_forward());
    }

    #[test]
    fn test_tab_transfer_data_json_serialization() {
        let tab_id = TabId::new();
        let window_id = WindowId::new();

        let data = TabTransferData::new(tab_id, window_id)
            .with_url("https://example.com")
            .with_title("Test");

        let json = data.to_json().unwrap();
        let restored = TabTransferData::from_json(&json).unwrap();

        assert_eq!(restored.url, data.url);
        assert_eq!(restored.title, data.title);
    }

    #[test]
    fn test_window_drop_target_contains() {
        let window_id = WindowId::new();
        let target = create_test_drop_target(window_id);

        assert!(target.contains(&Position::new(400.0, 300.0)));
        assert!(!target.contains(&Position::new(900.0, 300.0)));
    }

    #[test]
    fn test_window_drop_target_tab_bar_contains() {
        let window_id = WindowId::new();
        let target = create_test_drop_target(window_id);

        assert!(target.tab_bar_contains(&Position::new(400.0, 20.0)));
        assert!(!target.tab_bar_contains(&Position::new(400.0, 100.0)));
    }

    #[test]
    fn test_window_drop_target_calculate_drop_index() {
        let window_id = WindowId::new();
        let target = create_test_drop_target(window_id);

        // 5 tabs across 800px = 160px per tab
        assert_eq!(target.calculate_drop_index(&Position::new(80.0, 20.0)), 0);
        assert_eq!(target.calculate_drop_index(&Position::new(240.0, 20.0)), 1);
        assert_eq!(target.calculate_drop_index(&Position::new(750.0, 20.0)), 4);
    }

    #[test]
    fn test_drag_session_creation() {
        let tab_id = TabId::new();
        let window_id = WindowId::new();
        let position = Position::new(100.0, 50.0);
        let transfer_data = create_test_transfer_data(tab_id, window_id);

        let session = DragSession::new(tab_id, window_id, position, transfer_data);

        assert_eq!(session.tab_id, tab_id);
        assert_eq!(session.source_window_id, window_id);
        assert_eq!(session.state, TabDragState::Dragging);
    }

    #[test]
    fn test_drag_session_update_position() {
        let tab_id = TabId::new();
        let window_id = WindowId::new();
        let transfer_data = create_test_transfer_data(tab_id, window_id);

        let mut session = DragSession::new(
            tab_id,
            window_id,
            Position::new(0.0, 0.0),
            transfer_data,
        );

        session.update_position(Position::new(100.0, 100.0));
        assert_eq!(session.current_position.x, 100.0);
        assert_eq!(session.current_position.y, 100.0);
    }

    #[test]
    fn test_drag_session_hover_state() {
        let tab_id = TabId::new();
        let source_window_id = WindowId::new();
        let target_window_id = WindowId::new();
        let transfer_data = create_test_transfer_data(tab_id, source_window_id);

        let mut session = DragSession::new(
            tab_id,
            source_window_id,
            Position::new(0.0, 0.0),
            transfer_data,
        );

        assert_eq!(session.state, TabDragState::Dragging);
        assert!(session.hover_window_id.is_none());

        session.set_hover(Some(target_window_id));
        assert_eq!(session.state, TabDragState::Hovering);
        assert_eq!(session.hover_window_id, Some(target_window_id));

        session.set_hover(None);
        assert_eq!(session.state, TabDragState::Dragging);
    }

    #[test]
    fn test_drag_session_drag_distance() {
        let tab_id = TabId::new();
        let window_id = WindowId::new();
        let transfer_data = create_test_transfer_data(tab_id, window_id);

        let mut session = DragSession::new(
            tab_id,
            window_id,
            Position::new(0.0, 0.0),
            transfer_data,
        );

        session.update_position(Position::new(3.0, 4.0));
        assert!((session.drag_distance() - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_tab_drag_manager_new() {
        let manager = TabDragManager::new();
        assert!(!manager.is_dragging());
        assert_eq!(manager.get_state(), TabDragState::Idle);
    }

    #[test]
    fn test_tab_drag_manager_register_drop_target() {
        let mut manager = TabDragManager::new();
        let window_id = WindowId::new();
        let target = create_test_drop_target(window_id);

        manager.register_drop_target(target);

        assert!(manager.has_drop_target(window_id));
        assert_eq!(manager.drop_target_count(), 1);
    }

    #[test]
    fn test_tab_drag_manager_unregister_drop_target() {
        let mut manager = TabDragManager::new();
        let window_id = WindowId::new();
        let target = create_test_drop_target(window_id);

        manager.register_drop_target(target);
        assert!(manager.has_drop_target(window_id));

        manager.unregister_drop_target(window_id);
        assert!(!manager.has_drop_target(window_id));
    }

    #[test]
    fn test_tab_drag_manager_start_drag() {
        let mut manager = TabDragManager::new();
        let tab_id = TabId::new();
        let window_id = WindowId::new();
        let position = Position::new(100.0, 50.0);
        let transfer_data = create_test_transfer_data(tab_id, window_id);

        let result = manager.start_drag(tab_id, window_id, position, transfer_data);

        assert!(result.is_ok());
        assert!(manager.is_dragging());
        assert_eq!(manager.get_state(), TabDragState::Dragging);
    }

    #[test]
    fn test_tab_drag_manager_start_drag_already_in_progress() {
        let mut manager = TabDragManager::new();
        let tab_id = TabId::new();
        let window_id = WindowId::new();
        let transfer_data = create_test_transfer_data(tab_id, window_id);

        manager.start_drag(tab_id, window_id, Position::new(0.0, 0.0), transfer_data.clone()).unwrap();

        let result = manager.start_drag(TabId::new(), window_id, Position::new(0.0, 0.0), transfer_data);

        assert!(matches!(result, Err(TabDragError::DragAlreadyInProgress)));
    }

    #[test]
    fn test_tab_drag_manager_cancel_drag() {
        let mut manager = TabDragManager::new();
        let tab_id = TabId::new();
        let window_id = WindowId::new();
        let transfer_data = create_test_transfer_data(tab_id, window_id);

        manager.start_drag(tab_id, window_id, Position::new(0.0, 0.0), transfer_data).unwrap();

        let result = manager.cancel_drag();

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), tab_id);
        assert!(!manager.is_dragging());
    }

    #[test]
    fn test_tab_drag_manager_cancel_no_drag() {
        let mut manager = TabDragManager::new();
        let result = manager.cancel_drag();
        assert!(matches!(result, Err(TabDragError::NoDragInProgress)));
    }

    #[test]
    fn test_tab_drag_manager_update_position() {
        let mut manager = TabDragManager::new();
        let tab_id = TabId::new();
        let window_id = WindowId::new();
        let transfer_data = create_test_transfer_data(tab_id, window_id);

        manager.start_drag(tab_id, window_id, Position::new(0.0, 0.0), transfer_data).unwrap();

        let feedback = manager.update_drag_position(Position::new(100.0, 50.0));

        assert!(feedback.is_ok());
    }

    #[test]
    fn test_tab_drag_manager_complete_drop() {
        let mut manager = TabDragManager::new();
        let tab_id = TabId::new();
        let source_window_id = WindowId::new();
        let target_window_id = WindowId::new();
        let transfer_data = create_test_transfer_data(tab_id, source_window_id);

        // Register target window
        manager.register_drop_target(create_test_drop_target(target_window_id));

        // Start drag
        manager.start_drag(tab_id, source_window_id, Position::new(0.0, 0.0), transfer_data).unwrap();

        // Complete drop
        let result = manager.complete_drop(Some(target_window_id));

        assert!(result.is_ok());
        let (data, index) = result.unwrap();
        assert_eq!(data.tab_id, tab_id);
        assert!(index.is_some());
        assert!(!manager.is_dragging());
    }

    #[test]
    fn test_cross_window_message_json() {
        let msg = CrossWindowMessage::DragStarted {
            tab_id: TabId::new(),
            source_window_id: WindowId::new(),
            position: Position::new(100.0, 50.0),
        };

        let json = msg.to_json().unwrap();
        let restored = CrossWindowMessage::from_json(&json).unwrap();

        match restored {
            CrossWindowMessage::DragStarted { position, .. } => {
                assert_eq!(position.x, 100.0);
                assert_eq!(position.y, 50.0);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_drag_feedback_invalid() {
        let feedback = DragFeedback::invalid(Position::new(100.0, 200.0));

        assert!(!feedback.is_valid_drop);
        assert!(feedback.target_window_id.is_none());
        assert_eq!(feedback.indicator, DropIndicator::None);
    }

    #[test]
    fn test_drag_feedback_valid() {
        let window_id = WindowId::new();
        let feedback = DragFeedback::valid(Position::new(100.0, 200.0), window_id, 2);

        assert!(feedback.is_valid_drop);
        assert_eq!(feedback.target_window_id, Some(window_id));
        assert_eq!(feedback.suggested_drop_index, Some(2));
        assert_eq!(feedback.indicator, DropIndicator::InsertMarker);
    }

    #[test]
    fn test_take_outgoing_messages() {
        let mut manager = TabDragManager::new();
        let tab_id = TabId::new();
        let window_id = WindowId::new();
        let transfer_data = create_test_transfer_data(tab_id, window_id);

        manager.start_drag(tab_id, window_id, Position::new(0.0, 0.0), transfer_data).unwrap();

        let messages = manager.take_outgoing_messages();
        assert_eq!(messages.len(), 1);

        // Should be empty after take
        let messages = manager.take_outgoing_messages();
        assert!(messages.is_empty());
    }

    #[test]
    fn test_history_entry() {
        let entry = HistoryEntry::new("https://example.com", "Example");
        assert_eq!(entry.url, "https://example.com");
        assert_eq!(entry.title, "Example");
    }
}
