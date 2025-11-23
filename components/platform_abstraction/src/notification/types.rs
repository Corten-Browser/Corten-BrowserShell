//! Notification types and data structures
//!
//! This module defines the core types used for cross-platform system notifications.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Unique identifier for a notification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NotificationId(pub u64);

impl NotificationId {
    /// Create a new notification ID
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

impl From<u64> for NotificationId {
    fn from(id: u64) -> Self {
        Self(id)
    }
}

impl From<NotificationId> for u64 {
    fn from(id: NotificationId) -> Self {
        id.0
    }
}

/// Category of notification for proper system handling
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum NotificationCategory {
    /// General notification (default)
    #[default]
    General,
    /// Download completed notification
    DownloadComplete,
    /// Alert or warning notification
    Alert,
    /// Web page notification (from JavaScript Notification API)
    WebNotification,
    /// Error notification
    Error,
    /// Progress notification (for ongoing operations)
    Progress,
}

impl NotificationCategory {
    /// Get the hint category string for the notification system
    pub fn as_hint(&self) -> &'static str {
        match self {
            Self::General => "im.received",
            Self::DownloadComplete => "transfer.complete",
            Self::Alert => "device.warning",
            Self::WebNotification => "im.received",
            Self::Error => "device.error",
            Self::Progress => "transfer",
        }
    }
}

/// Urgency level for notifications
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum NotificationUrgency {
    /// Low urgency - can be shown at user's convenience
    Low,
    /// Normal urgency (default)
    #[default]
    Normal,
    /// Critical urgency - should interrupt user
    Critical,
}

/// Permission status for showing notifications
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NotificationPermission {
    /// Permission has not been requested yet
    Default,
    /// User has granted permission
    Granted,
    /// User has denied permission
    Denied,
}

impl Default for NotificationPermission {
    fn default() -> Self {
        Self::Default
    }
}

/// Action button that can be displayed on a notification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotificationAction {
    /// Unique identifier for the action
    pub id: String,
    /// Display text for the action button
    pub label: String,
}

impl NotificationAction {
    /// Create a new notification action
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
        }
    }
}

/// Configuration settings for the notification service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    /// Application name shown in notifications
    pub app_name: String,
    /// Default icon path for notifications
    pub default_icon: Option<PathBuf>,
    /// Whether to request permission on first use
    pub auto_request_permission: bool,
    /// Maximum number of actions per notification (platform-dependent)
    pub max_actions: usize,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            app_name: String::from("Browser Shell"),
            default_icon: None,
            auto_request_permission: true,
            max_actions: 3,
        }
    }
}

/// Event emitted when user interacts with a notification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationEvent {
    /// Notification was clicked (body clicked)
    Clicked(NotificationId),
    /// Notification was closed by user or timeout
    Closed(NotificationId),
    /// Action button was clicked
    ActionClicked {
        notification_id: NotificationId,
        action_id: String,
    },
}

/// A system notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    /// Title of the notification
    pub title: String,
    /// Body text of the notification
    pub body: String,
    /// Optional icon path
    pub icon: Option<PathBuf>,
    /// Action buttons
    pub actions: Vec<NotificationAction>,
    /// Notification category
    pub category: NotificationCategory,
    /// Urgency level
    pub urgency: NotificationUrgency,
    /// Whether notification should persist until dismissed
    pub persistent: bool,
    /// Optional timeout in milliseconds (0 = use system default)
    pub timeout_ms: Option<u32>,
    /// Optional tag for replacing existing notifications
    pub tag: Option<String>,
}

impl Notification {
    /// Create a new notification with the given title
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            body: String::new(),
            icon: None,
            actions: Vec::new(),
            category: NotificationCategory::default(),
            urgency: NotificationUrgency::default(),
            persistent: false,
            timeout_ms: None,
            tag: None,
        }
    }

    /// Create a notification builder for fluent construction
    pub fn builder(title: impl Into<String>) -> super::NotificationBuilder {
        super::NotificationBuilder::new(title)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_id_creation() {
        let id = NotificationId::new(42);
        assert_eq!(id.0, 42);
    }

    #[test]
    fn test_notification_id_from_u64() {
        let id: NotificationId = 123.into();
        assert_eq!(id.0, 123);
    }

    #[test]
    fn test_notification_id_to_u64() {
        let id = NotificationId::new(456);
        let value: u64 = id.into();
        assert_eq!(value, 456);
    }

    #[test]
    fn test_notification_category_hint() {
        assert_eq!(NotificationCategory::General.as_hint(), "im.received");
        assert_eq!(
            NotificationCategory::DownloadComplete.as_hint(),
            "transfer.complete"
        );
        assert_eq!(NotificationCategory::Alert.as_hint(), "device.warning");
        assert_eq!(NotificationCategory::Error.as_hint(), "device.error");
    }

    #[test]
    fn test_notification_action_creation() {
        let action = NotificationAction::new("open", "Open File");
        assert_eq!(action.id, "open");
        assert_eq!(action.label, "Open File");
    }

    #[test]
    fn test_notification_config_default() {
        let config = NotificationConfig::default();
        assert_eq!(config.app_name, "Browser Shell");
        assert!(config.default_icon.is_none());
        assert!(config.auto_request_permission);
        assert_eq!(config.max_actions, 3);
    }

    #[test]
    fn test_notification_new() {
        let notification = Notification::new("Test Title");
        assert_eq!(notification.title, "Test Title");
        assert_eq!(notification.body, "");
        assert!(notification.icon.is_none());
        assert!(notification.actions.is_empty());
        assert_eq!(notification.category, NotificationCategory::General);
        assert_eq!(notification.urgency, NotificationUrgency::Normal);
        assert!(!notification.persistent);
    }

    #[test]
    fn test_notification_event_variants() {
        let id = NotificationId::new(1);

        let clicked = NotificationEvent::Clicked(id);
        assert!(matches!(clicked, NotificationEvent::Clicked(_)));

        let closed = NotificationEvent::Closed(id);
        assert!(matches!(closed, NotificationEvent::Closed(_)));

        let action = NotificationEvent::ActionClicked {
            notification_id: id,
            action_id: "test".to_string(),
        };
        assert!(matches!(action, NotificationEvent::ActionClicked { .. }));
    }

    #[test]
    fn test_notification_permission_default() {
        let permission = NotificationPermission::default();
        assert_eq!(permission, NotificationPermission::Default);
    }
}
