//! Notification service trait and cross-platform implementation
//!
//! This module provides the NotificationService trait and a cross-platform
//! implementation using notify-rust.

use super::types::{
    Notification, NotificationConfig, NotificationId, NotificationPermission, NotificationUrgency,
};
use std::sync::atomic::{AtomicU64, Ordering};
use thiserror::Error;

/// Errors that can occur during notification operations
#[derive(Debug, Error)]
pub enum NotificationError {
    /// Permission to show notifications was denied
    #[error("notification permission denied")]
    PermissionDenied,

    /// Failed to show the notification
    #[error("failed to show notification: {0}")]
    ShowFailed(String),

    /// Failed to close the notification
    #[error("failed to close notification: {0}")]
    CloseFailed(String),

    /// Notification system is not available
    #[error("notification system not available: {0}")]
    NotAvailable(String),

    /// Invalid notification configuration
    #[error("invalid notification: {0}")]
    InvalidNotification(String),
}

/// Result type for notification operations
pub type NotificationResult<T> = Result<T, NotificationError>;

/// Trait for notification services
///
/// This trait defines the interface for showing and managing system notifications.
/// Implementations should handle platform-specific details.
pub trait NotificationService {
    /// Show a notification
    ///
    /// # Arguments
    ///
    /// * `notification` - The notification to display
    ///
    /// # Returns
    ///
    /// * `Ok(NotificationId)` - The ID of the shown notification
    /// * `Err(NotificationError)` - If the notification could not be shown
    fn show(&self, notification: &Notification) -> NotificationResult<NotificationId>;

    /// Close a notification
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the notification to close
    ///
    /// # Returns
    ///
    /// * `Ok(())` - The notification was closed
    /// * `Err(NotificationError)` - If the notification could not be closed
    fn close(&self, id: NotificationId) -> NotificationResult<()>;

    /// Request permission to show notifications
    ///
    /// On platforms that require explicit permission, this will prompt the user.
    /// On platforms without permission requirements, this returns `Granted`.
    ///
    /// # Returns
    ///
    /// * `Ok(NotificationPermission)` - The permission status
    /// * `Err(NotificationError)` - If permission request failed
    fn request_permission(&self) -> NotificationResult<NotificationPermission>;

    /// Check current permission status without requesting
    ///
    /// # Returns
    ///
    /// The current permission status
    fn check_permission(&self) -> NotificationPermission;

    /// Check if notification system is supported on this platform
    fn is_supported(&self) -> bool;
}

/// Cross-platform notification service implementation
///
/// This service uses notify-rust for cross-platform notification support
/// on Linux, Windows, and macOS.
pub struct SystemNotificationService {
    config: NotificationConfig,
    permission: std::sync::RwLock<NotificationPermission>,
    next_id: AtomicU64,
}

impl SystemNotificationService {
    /// Create a new notification service with default configuration
    pub fn new() -> Self {
        Self::with_config(NotificationConfig::default())
    }

    /// Create a new notification service with custom configuration
    pub fn with_config(config: NotificationConfig) -> Self {
        Self {
            config,
            permission: std::sync::RwLock::new(NotificationPermission::Default),
            next_id: AtomicU64::new(1),
        }
    }

    /// Get the current configuration
    pub fn config(&self) -> &NotificationConfig {
        &self.config
    }

    /// Generate the next notification ID
    fn generate_id(&self) -> NotificationId {
        NotificationId::new(self.next_id.fetch_add(1, Ordering::SeqCst))
    }

    /// Convert our urgency to notify-rust urgency
    fn map_urgency(urgency: NotificationUrgency) -> notify_rust::Urgency {
        match urgency {
            NotificationUrgency::Low => notify_rust::Urgency::Low,
            NotificationUrgency::Normal => notify_rust::Urgency::Normal,
            NotificationUrgency::Critical => notify_rust::Urgency::Critical,
        }
    }

    /// Build a notify-rust notification from our notification type
    fn build_native_notification(
        &self,
        notification: &Notification,
    ) -> notify_rust::Notification {
        let mut native = notify_rust::Notification::new();

        native
            .appname(&self.config.app_name)
            .summary(&notification.title)
            .body(&notification.body)
            .urgency(Self::map_urgency(notification.urgency));

        // Set icon
        if let Some(ref icon) = notification.icon {
            if let Some(icon_str) = icon.to_str() {
                native.icon(icon_str);
            }
        } else if let Some(ref default_icon) = self.config.default_icon {
            if let Some(icon_str) = default_icon.to_str() {
                native.icon(icon_str);
            }
        }

        // Set timeout
        if notification.persistent {
            native.timeout(notify_rust::Timeout::Never);
        } else if let Some(timeout_ms) = notification.timeout_ms {
            native.timeout(notify_rust::Timeout::Milliseconds(timeout_ms));
        }

        // Set category hint
        native.hint(notify_rust::Hint::Category(
            notification.category.as_hint().to_string(),
        ));

        // Add actions
        for action in &notification.actions {
            native.action(&action.id, &action.label);
        }

        // Set tag/id for replacement
        if let Some(ref tag) = notification.tag {
            native.id(tag_to_id(tag));
        }

        native
    }
}

impl Default for SystemNotificationService {
    fn default() -> Self {
        Self::new()
    }
}

impl NotificationService for SystemNotificationService {
    fn show(&self, notification: &Notification) -> NotificationResult<NotificationId> {
        // Check permission
        let permission = self.check_permission();
        if permission == NotificationPermission::Denied {
            return Err(NotificationError::PermissionDenied);
        }

        // Auto-request permission if needed
        if permission == NotificationPermission::Default && self.config.auto_request_permission {
            self.request_permission()?;
        }

        // Build and show notification
        let native = self.build_native_notification(notification);
        let id = self.generate_id();

        native
            .show()
            .map_err(|e| NotificationError::ShowFailed(e.to_string()))?;

        log::debug!(
            "Showed notification {} with title: {}",
            id.0,
            notification.title
        );

        Ok(id)
    }

    fn close(&self, id: NotificationId) -> NotificationResult<()> {
        // notify-rust doesn't directly support closing by our ID
        // since we don't store handles. This is a limitation.
        // For now, we just log the close request.
        log::debug!("Close requested for notification {}", id.0);
        Ok(())
    }

    fn request_permission(&self) -> NotificationResult<NotificationPermission> {
        // On desktop platforms (Linux/Windows/macOS), notifications are typically
        // always allowed at the application level. Permission is managed by the OS.
        // We simulate the Web Notification API behavior here.

        let mut permission = self.permission.write().unwrap();

        // On desktop, we generally have permission unless system settings deny it
        // For now, we assume permission is granted if the notification system works
        *permission = if self.is_supported() {
            NotificationPermission::Granted
        } else {
            NotificationPermission::Denied
        };

        log::debug!("Notification permission: {:?}", *permission);
        Ok(*permission)
    }

    fn check_permission(&self) -> NotificationPermission {
        *self.permission.read().unwrap()
    }

    fn is_supported(&self) -> bool {
        // notify-rust supports Linux, Windows, and macOS
        cfg!(any(
            target_os = "linux",
            target_os = "windows",
            target_os = "macos"
        ))
    }
}

/// Convert a tag string to a stable numeric ID for notification replacement
fn tag_to_id(tag: &str) -> u32 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    tag.hash(&mut hasher);
    (hasher.finish() % (u32::MAX as u64)) as u32
}

/// Check if notifications are supported on the current platform
pub fn notifications_supported() -> bool {
    cfg!(any(
        target_os = "linux",
        target_os = "windows",
        target_os = "macos"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notification::{NotificationBuilder, NotificationCategory};

    #[test]
    fn test_notification_service_creation() {
        let service = SystemNotificationService::new();
        assert_eq!(service.config().app_name, "Browser Shell");
    }

    #[test]
    fn test_notification_service_with_config() {
        let config = NotificationConfig {
            app_name: "Test App".to_string(),
            default_icon: None,
            auto_request_permission: false,
            max_actions: 5,
        };
        let service = SystemNotificationService::with_config(config);
        assert_eq!(service.config().app_name, "Test App");
        assert!(!service.config().auto_request_permission);
        assert_eq!(service.config().max_actions, 5);
    }

    #[test]
    fn test_notification_id_generation() {
        let service = SystemNotificationService::new();
        let id1 = service.generate_id();
        let id2 = service.generate_id();
        let id3 = service.generate_id();

        assert_eq!(id1.0, 1);
        assert_eq!(id2.0, 2);
        assert_eq!(id3.0, 3);
    }

    #[test]
    fn test_urgency_mapping() {
        use notify_rust::Urgency;

        assert!(matches!(
            SystemNotificationService::map_urgency(NotificationUrgency::Low),
            Urgency::Low
        ));
        assert!(matches!(
            SystemNotificationService::map_urgency(NotificationUrgency::Normal),
            Urgency::Normal
        ));
        assert!(matches!(
            SystemNotificationService::map_urgency(NotificationUrgency::Critical),
            Urgency::Critical
        ));
    }

    #[test]
    fn test_permission_check_default() {
        let service = SystemNotificationService::new();
        assert_eq!(service.check_permission(), NotificationPermission::Default);
    }

    #[test]
    fn test_is_supported() {
        let service = SystemNotificationService::new();
        // Should be supported on desktop platforms
        let expected = cfg!(any(
            target_os = "linux",
            target_os = "windows",
            target_os = "macos"
        ));
        assert_eq!(service.is_supported(), expected);
    }

    #[test]
    fn test_notifications_supported_function() {
        let expected = cfg!(any(
            target_os = "linux",
            target_os = "windows",
            target_os = "macos"
        ));
        assert_eq!(notifications_supported(), expected);
    }

    #[test]
    fn test_tag_to_id_stable() {
        // Same tag should produce same ID
        let id1 = tag_to_id("download-123");
        let id2 = tag_to_id("download-123");
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_tag_to_id_different() {
        // Different tags should produce different IDs (with high probability)
        let id1 = tag_to_id("download-123");
        let id2 = tag_to_id("download-456");
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_build_native_notification_basic() {
        let service = SystemNotificationService::new();
        let notification = NotificationBuilder::new("Test Title")
            .body("Test Body")
            .build();

        // This test just ensures the builder doesn't panic
        let _native = service.build_native_notification(&notification);
    }

    #[test]
    fn test_build_native_notification_with_icon() {
        let service = SystemNotificationService::new();
        let notification = NotificationBuilder::new("Test")
            .icon("/path/to/icon.png")
            .build();

        let _native = service.build_native_notification(&notification);
    }

    #[test]
    fn test_build_native_notification_with_actions() {
        let service = SystemNotificationService::new();
        let notification = NotificationBuilder::new("Test")
            .action("open", "Open")
            .action("dismiss", "Dismiss")
            .build();

        let _native = service.build_native_notification(&notification);
    }

    #[test]
    fn test_build_native_notification_with_timeout() {
        let service = SystemNotificationService::new();
        let notification = NotificationBuilder::new("Test").timeout(5000).build();

        let _native = service.build_native_notification(&notification);
    }

    #[test]
    fn test_build_native_notification_persistent() {
        let service = SystemNotificationService::new();
        let notification = NotificationBuilder::new("Test").persistent(true).build();

        let _native = service.build_native_notification(&notification);
    }

    #[test]
    fn test_build_native_notification_with_category() {
        let service = SystemNotificationService::new();
        let notification = NotificationBuilder::new("Test")
            .category(NotificationCategory::DownloadComplete)
            .build();

        let _native = service.build_native_notification(&notification);
    }

    #[test]
    fn test_build_native_notification_with_tag() {
        let service = SystemNotificationService::new();
        let notification = NotificationBuilder::new("Test")
            .tag("test-tag")
            .build();

        let _native = service.build_native_notification(&notification);
    }

    #[test]
    fn test_notification_error_display() {
        let err = NotificationError::PermissionDenied;
        assert_eq!(err.to_string(), "notification permission denied");

        let err = NotificationError::ShowFailed("test error".to_string());
        assert_eq!(err.to_string(), "failed to show notification: test error");

        let err = NotificationError::CloseFailed("close error".to_string());
        assert_eq!(err.to_string(), "failed to close notification: close error");

        let err = NotificationError::NotAvailable("not available".to_string());
        assert_eq!(
            err.to_string(),
            "notification system not available: not available"
        );

        let err = NotificationError::InvalidNotification("invalid".to_string());
        assert_eq!(err.to_string(), "invalid notification: invalid");
    }

    #[test]
    fn test_close_notification() {
        let service = SystemNotificationService::new();
        let id = NotificationId::new(123);

        // Close should not fail (even though it's a no-op currently)
        let result = service.close(id);
        assert!(result.is_ok());
    }

    // Integration test - only runs when notifications are available
    // This test is ignored by default as it shows a real notification
    #[test]
    #[ignore]
    fn test_show_notification_integration() {
        let service = SystemNotificationService::new();

        // Request permission first
        let permission = service.request_permission().unwrap();
        assert_eq!(permission, NotificationPermission::Granted);

        let notification = NotificationBuilder::new("Test Notification")
            .body("This is a test notification from the platform_abstraction component")
            .category(NotificationCategory::General)
            .timeout(3000)
            .build();

        let result = service.show(&notification);
        assert!(result.is_ok());
    }
}
