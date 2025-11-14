//! Message types for component communication
//!
//! This module defines the message types used for inter-component communication
//! in the browser shell, including commands, responses, and priority levels.

use serde::{Deserialize, Serialize};
use shared_types::{KeyboardShortcut, TabId, WindowConfig, WindowId};

/// Messages that can be sent between components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentMessage {
    /// Create a new browser window with the specified configuration
    CreateWindow(WindowConfig),

    /// Close the specified window
    CloseWindow(WindowId),

    /// Create a new tab in the specified window, optionally with a URL
    CreateTab(WindowId, Option<String>),

    /// Close the specified tab
    CloseTab(TabId),

    /// Navigate the specified tab to a URL
    NavigateTab(TabId, String),

    /// Update the address bar for the specified tab
    UpdateAddressBar(TabId, String),

    /// Update the title for the specified tab
    UpdateTitle(TabId, String),

    /// Handle a keyboard shortcut
    KeyboardShortcut(KeyboardShortcut),
}

/// Responses to component messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentResponse {
    /// A window was successfully created
    WindowCreated(WindowId),

    /// A tab was successfully created
    TabCreated(TabId),

    /// Navigation has started for the specified tab
    NavigationStarted(TabId),

    /// The operation completed successfully
    Success,

    /// An error occurred during the operation
    Error(String),
}

/// Priority levels for message processing
///
/// Higher priority messages are processed first in the message queue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MessagePriority {
    /// Lowest priority - background tasks
    Low,

    /// Normal priority - standard operations
    Normal,

    /// High priority - user-initiated actions
    High,

    /// Highest priority - critical system operations
    Critical,
}

impl Default for MessagePriority {
    fn default() -> Self {
        Self::Normal
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_priority_ordering() {
        assert!(MessagePriority::Critical > MessagePriority::High);
        assert!(MessagePriority::High > MessagePriority::Normal);
        assert!(MessagePriority::Normal > MessagePriority::Low);
    }

    #[test]
    fn test_message_priority_default() {
        assert_eq!(MessagePriority::default(), MessagePriority::Normal);
    }
}
