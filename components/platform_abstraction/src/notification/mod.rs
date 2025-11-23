//! Cross-platform system notification support
//!
//! This module provides a unified API for showing system notifications across
//! Linux, Windows, and macOS platforms.
//!
//! # Overview
//!
//! The notification system provides:
//! - Cross-platform notification API using notify-rust
//! - Support for notification title, body, and icon
//! - Action buttons on notifications
//! - Notification categories (download complete, alert, web notification, etc.)
//! - Permission checking following Web Notification API patterns
//! - Fluent builder API for easy notification construction
//!
//! # Example
//!
//! ```rust,no_run
//! use platform_abstraction::notification::{
//!     NotificationBuilder, NotificationService, SystemNotificationService,
//!     NotificationCategory,
//! };
//!
//! // Create a notification service
//! let service = SystemNotificationService::new();
//!
//! // Request permission (on desktop platforms, this typically succeeds)
//! let permission = service.request_permission().unwrap();
//!
//! // Build and show a notification
//! let notification = NotificationBuilder::new("Download Complete")
//!     .body("file.zip has finished downloading")
//!     .category(NotificationCategory::DownloadComplete)
//!     .action("open", "Open File")
//!     .action("folder", "Show in Folder")
//!     .build();
//!
//! let id = service.show(&notification).unwrap();
//! ```
//!
//! # Convenience Builders
//!
//! The builder provides convenience methods for common notification types:
//!
//! ```rust,no_run
//! use platform_abstraction::notification::NotificationBuilder;
//!
//! // Download complete notification with standard actions
//! let download = NotificationBuilder::download_complete(
//!     "Download Complete",
//!     "/path/to/file.zip"
//! ).build();
//!
//! // Alert notification with critical urgency
//! let alert = NotificationBuilder::alert(
//!     "Warning",
//!     "Disk space is running low"
//! ).build();
//!
//! // Error notification
//! let error = NotificationBuilder::error(
//!     "Connection Error",
//!     "Failed to connect to server"
//! ).build();
//!
//! // Web notification (from JavaScript Notification API)
//! let web = NotificationBuilder::web_notification(
//!     "New Message",
//!     "You have a new message",
//!     None
//! ).build();
//! ```

mod builder;
mod service;
mod types;

// Re-export public types
pub use builder::NotificationBuilder;
pub use service::{
    notifications_supported, NotificationError, NotificationResult, NotificationService,
    SystemNotificationService,
};
pub use types::{
    Notification, NotificationAction, NotificationCategory, NotificationConfig, NotificationEvent,
    NotificationId, NotificationPermission, NotificationUrgency,
};
