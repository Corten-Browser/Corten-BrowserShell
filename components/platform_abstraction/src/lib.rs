//! Platform abstraction layer for browser shell window management
//!
//! This crate provides platform-specific window implementations for Linux, Windows, and macOS.
//! It exposes a common `PlatformWindow` trait that all platform implementations must satisfy,
//! along with platform-specific handle types.
//!
//! # Features
//!
//! - **Window Management**: Cross-platform window creation, manipulation, and lifecycle management
//! - **System Notifications**: Cross-platform notification support with actions and categories
//! - **Clipboard**: Cross-platform clipboard support for text, HTML, and images
//! - **File Associations**: Cross-platform file type and protocol registration
//! - **Drag and Drop**: Cross-platform drag and drop support for files, text, HTML, images, and URLs
//!
//! # Phase 1: Stub Implementation
//!
//! This is a stub implementation that compiles on all platforms and provides mock functionality.
//! Full native window integration will be implemented in later phases.

pub mod clipboard;
pub mod drag_drop;
pub mod file_associations;
mod handles;
pub mod notification;
mod platform;
mod traits;

// Re-export public types
pub use handles::*;
pub use platform::*;
pub use traits::*;

// Re-export notification types at top level for convenience
pub use notification::{
    Notification, NotificationAction, NotificationBuilder, NotificationCategory, NotificationConfig,
    NotificationError, NotificationEvent, NotificationId, NotificationPermission, NotificationResult,
    NotificationService, NotificationUrgency, SystemNotificationService,
};

// Re-export clipboard types at top level for convenience
pub use clipboard::{
    clipboard_supported, ClipboardContent, ClipboardError, ClipboardFormat, ClipboardResult,
    ClipboardService, ImageData, SystemClipboardService,
};

// Re-export file association types at top level for convenience
pub use file_associations::{
    associations_supported, AssociationConfig, AssociationError, AssociationResult,
    AssociationService, AssociationStatus, FileAssociation, SystemAssociationService,
};

// Re-export drag and drop types at top level for convenience
pub use drag_drop::{
    drag_drop_supported, DragData, DragDropError, DragDropEvent, DragDropManager, DragDropResult,
    DragFormat, DragSource, DragState, DropEffect, DropIndicator, DropIndicatorStyle, DropTarget,
    DropTargetId, FileDropTarget, Point, TextDragSource, UrlDragSource,
};
