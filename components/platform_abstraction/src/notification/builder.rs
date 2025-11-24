//! Notification builder with fluent API
//!
//! This module provides a builder pattern for constructing notifications
//! with a clean, chainable API.

use super::types::{
    Notification, NotificationAction, NotificationCategory, NotificationUrgency,
};
use std::path::PathBuf;

/// Builder for constructing notifications with a fluent API
#[derive(Debug, Clone)]
pub struct NotificationBuilder {
    notification: Notification,
}

impl NotificationBuilder {
    /// Create a new builder with the given title
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            notification: Notification::new(title),
        }
    }

    /// Set the body text of the notification
    pub fn body(mut self, body: impl Into<String>) -> Self {
        self.notification.body = body.into();
        self
    }

    /// Set the icon path for the notification
    pub fn icon(mut self, icon: impl Into<PathBuf>) -> Self {
        self.notification.icon = Some(icon.into());
        self
    }

    /// Add an action button to the notification
    pub fn action(mut self, id: impl Into<String>, label: impl Into<String>) -> Self {
        self.notification
            .actions
            .push(NotificationAction::new(id, label));
        self
    }

    /// Add a pre-built action to the notification
    pub fn add_action(mut self, action: NotificationAction) -> Self {
        self.notification.actions.push(action);
        self
    }

    /// Set the notification category
    pub fn category(mut self, category: NotificationCategory) -> Self {
        self.notification.category = category;
        self
    }

    /// Set the notification urgency level
    pub fn urgency(mut self, urgency: NotificationUrgency) -> Self {
        self.notification.urgency = urgency;
        self
    }

    /// Mark notification as persistent (won't auto-dismiss)
    pub fn persistent(mut self, persistent: bool) -> Self {
        self.notification.persistent = persistent;
        self
    }

    /// Set the timeout in milliseconds
    pub fn timeout(mut self, timeout_ms: u32) -> Self {
        self.notification.timeout_ms = Some(timeout_ms);
        self
    }

    /// Set a tag for replacing existing notifications with the same tag
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.notification.tag = Some(tag.into());
        self
    }

    /// Build the notification
    pub fn build(self) -> Notification {
        self.notification
    }
}

// Convenience methods for common notification types
impl NotificationBuilder {
    /// Create a download complete notification
    pub fn download_complete(title: impl Into<String>, file_path: impl Into<String>) -> Self {
        Self::new(title)
            .body(file_path)
            .category(NotificationCategory::DownloadComplete)
            .action("open", "Open")
            .action("show", "Show in Folder")
    }

    /// Create an alert notification
    pub fn alert(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(title)
            .body(message)
            .category(NotificationCategory::Alert)
            .urgency(NotificationUrgency::Critical)
    }

    /// Create an error notification
    pub fn error(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(title)
            .body(message)
            .category(NotificationCategory::Error)
            .urgency(NotificationUrgency::Critical)
    }

    /// Create a web notification (from JavaScript Notification API)
    pub fn web_notification(
        title: impl Into<String>,
        body: impl Into<String>,
        icon: Option<PathBuf>,
    ) -> Self {
        let mut builder = Self::new(title)
            .body(body)
            .category(NotificationCategory::WebNotification);

        if let Some(icon_path) = icon {
            builder = builder.icon(icon_path);
        }

        builder
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_basic() {
        let notification = NotificationBuilder::new("Test Title")
            .body("Test Body")
            .build();

        assert_eq!(notification.title, "Test Title");
        assert_eq!(notification.body, "Test Body");
    }

    #[test]
    fn test_builder_with_icon() {
        let notification = NotificationBuilder::new("Test")
            .icon("/path/to/icon.png")
            .build();

        assert_eq!(
            notification.icon,
            Some(PathBuf::from("/path/to/icon.png"))
        );
    }

    #[test]
    fn test_builder_with_actions() {
        let notification = NotificationBuilder::new("Test")
            .action("open", "Open")
            .action("dismiss", "Dismiss")
            .build();

        assert_eq!(notification.actions.len(), 2);
        assert_eq!(notification.actions[0].id, "open");
        assert_eq!(notification.actions[0].label, "Open");
        assert_eq!(notification.actions[1].id, "dismiss");
        assert_eq!(notification.actions[1].label, "Dismiss");
    }

    #[test]
    fn test_builder_with_category() {
        let notification = NotificationBuilder::new("Test")
            .category(NotificationCategory::DownloadComplete)
            .build();

        assert_eq!(notification.category, NotificationCategory::DownloadComplete);
    }

    #[test]
    fn test_builder_with_urgency() {
        let notification = NotificationBuilder::new("Test")
            .urgency(NotificationUrgency::Critical)
            .build();

        assert_eq!(notification.urgency, NotificationUrgency::Critical);
    }

    #[test]
    fn test_builder_persistent() {
        let notification = NotificationBuilder::new("Test")
            .persistent(true)
            .build();

        assert!(notification.persistent);
    }

    #[test]
    fn test_builder_timeout() {
        let notification = NotificationBuilder::new("Test")
            .timeout(5000)
            .build();

        assert_eq!(notification.timeout_ms, Some(5000));
    }

    #[test]
    fn test_builder_tag() {
        let notification = NotificationBuilder::new("Test")
            .tag("download-1")
            .build();

        assert_eq!(notification.tag, Some("download-1".to_string()));
    }

    #[test]
    fn test_builder_fluent_chain() {
        let notification = NotificationBuilder::new("Download Complete")
            .body("file.zip downloaded successfully")
            .icon("/icons/download.png")
            .category(NotificationCategory::DownloadComplete)
            .urgency(NotificationUrgency::Normal)
            .action("open", "Open File")
            .action("folder", "Show in Folder")
            .timeout(10000)
            .tag("download-123")
            .build();

        assert_eq!(notification.title, "Download Complete");
        assert_eq!(notification.body, "file.zip downloaded successfully");
        assert_eq!(
            notification.icon,
            Some(PathBuf::from("/icons/download.png"))
        );
        assert_eq!(notification.category, NotificationCategory::DownloadComplete);
        assert_eq!(notification.urgency, NotificationUrgency::Normal);
        assert_eq!(notification.actions.len(), 2);
        assert_eq!(notification.timeout_ms, Some(10000));
        assert_eq!(notification.tag, Some("download-123".to_string()));
    }

    #[test]
    fn test_download_complete_convenience() {
        let notification =
            NotificationBuilder::download_complete("Download Complete", "/path/to/file.zip")
                .build();

        assert_eq!(notification.title, "Download Complete");
        assert_eq!(notification.body, "/path/to/file.zip");
        assert_eq!(notification.category, NotificationCategory::DownloadComplete);
        assert_eq!(notification.actions.len(), 2);
    }

    #[test]
    fn test_alert_convenience() {
        let notification = NotificationBuilder::alert("Warning", "Disk space low").build();

        assert_eq!(notification.title, "Warning");
        assert_eq!(notification.body, "Disk space low");
        assert_eq!(notification.category, NotificationCategory::Alert);
        assert_eq!(notification.urgency, NotificationUrgency::Critical);
    }

    #[test]
    fn test_error_convenience() {
        let notification =
            NotificationBuilder::error("Error", "Failed to connect").build();

        assert_eq!(notification.title, "Error");
        assert_eq!(notification.body, "Failed to connect");
        assert_eq!(notification.category, NotificationCategory::Error);
        assert_eq!(notification.urgency, NotificationUrgency::Critical);
    }

    #[test]
    fn test_web_notification_convenience() {
        let notification = NotificationBuilder::web_notification(
            "New Message",
            "You have a new message",
            Some(PathBuf::from("/favicon.ico")),
        )
        .build();

        assert_eq!(notification.title, "New Message");
        assert_eq!(notification.body, "You have a new message");
        assert_eq!(notification.category, NotificationCategory::WebNotification);
        assert_eq!(
            notification.icon,
            Some(PathBuf::from("/favicon.ico"))
        );
    }

    #[test]
    fn test_web_notification_without_icon() {
        let notification =
            NotificationBuilder::web_notification("Test", "Body", None).build();

        assert!(notification.icon.is_none());
    }
}
