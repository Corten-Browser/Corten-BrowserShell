//! Platform abstraction layer for browser shell window management
//!
//! This crate provides platform-specific window implementations for Linux, Windows, and macOS.
//! It exposes a common `PlatformWindow` trait that all platform implementations must satisfy,
//! along with platform-specific handle types and detection capabilities.
//!
//! # Features
//!
//! - **Window Management**: Cross-platform window creation, manipulation, and lifecycle management
//! - **Platform Detection**: Runtime detection of OS and display server (X11, Wayland, Win32, Cocoa)
//! - **System Notifications**: Cross-platform notification support with actions and categories
//! - **Clipboard**: Cross-platform clipboard support for text, HTML, and images
//! - **File Associations**: Cross-platform file type and protocol registration
//! - **Drag and Drop**: Cross-platform drag and drop support for files, text, HTML, images, and URLs
//!
//! # Platform Support
//!
//! - **Linux X11**: Full window management via x11rb-style API
//! - **Linux Wayland**: Full window management via wayland-client-style API
//! - **Windows**: Full window management via windows-rs-style API (Win32)
//! - **macOS**: Full window management via cocoa-style API (AppKit)
//!
//! # Usage
//!
//! ```rust,ignore
//! use platform_abstraction::{
//!     platform::{create_platform_window, current_platform, detect_display_server},
//!     PlatformWindow,
//! };
//! use shared_types::WindowConfig;
//!
//! // Auto-detect platform and create appropriate window
//! let config = WindowConfig::default();
//! let mut window = create_platform_window(&config)?;
//!
//! // Use the window
//! window.show()?;
//! window.resize(800, 600)?;
//! window.focus()?;
//!
//! // Clean up
//! window.destroy()?;
//! ```

pub mod clipboard;
pub mod drag_drop;
pub mod file_associations;
mod handles;
pub mod notification;
pub mod platform;
mod traits;

// Re-export handle types
pub use handles::*;

// Re-export trait
pub use traits::*;

// Re-export platform module types at top level for convenience
pub use platform::{
    create_platform_window, create_window_for_display_server, current_platform,
    detect_display_server, get_platform_info, preferred_display_server, supports_display_server,
    DisplayServer, GenericPlatformWindow, Platform, PlatformDetails, PlatformInfo, StubWindow,
};

// Re-export platform-specific window types
pub use platform::{
    // Linux X11
    LinuxX11Window, WindowConfigX11, X11Atoms, X11Geometry, X11WindowAttributes, X11WindowState,
    // Linux Wayland
    DecorationMode, LinuxWaylandWindow, ToplevelCapabilities, WaylandConfigureState,
    WaylandGeometry, WaylandSurfaceConfig, WaylandTiledState, WaylandWindowState,
    // Windows
    DwmAttributes, WindowRect, WindowStyle, WindowsWindow, WindowsWindowState,
    // macOS
    AppearanceMode, BackingStoreType, CollectionBehavior, MacWindow, MacWindowState, NSRect,
    WindowStyleMask,
};

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
