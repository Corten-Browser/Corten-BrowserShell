// @implements: REQ-001, REQ-002, REQ-003, REQ-004, REQ-005, REQ-006
//! Shared Types Library
//!
//! Common types, interfaces, and message protocol definitions for all browser components.
//!
//! # Modules
//!
//! - `error`: Error types used across components
//! - `component`: Base component interface and types
//! - `window`: Window management types
//! - `tab`: Tab management types
//! - `message`: Inter-component message protocol
//!
//! # Example
//!
//! ```rust
//! use shared_types::tab::{TabId, Url};
//! use shared_types::window::WindowId;
//! use shared_types::component::ComponentHealth;
//!
//! let tab_id = TabId::new();
//! let window_id = WindowId::new();
//! let url = Url::parse("https://example.com").expect("Valid URL");
//! let health = ComponentHealth::Healthy;
//! ```

pub mod error;
pub mod component;
pub mod window;
pub mod tab;
pub mod message;

// Re-export commonly used types
pub use error::{ComponentError, TabError, WindowError};
pub use component::{
    BrowserComponent, ComponentHealth, ComponentMetrics, ComponentConfig,
    ComponentMessage, ComponentResponse, MessageTarget, MessagePriority,
    MessagePayload, LogLevel,
};
pub use window::{
    WindowId, WindowConfig, WindowUpdate, PlatformEvent, Window, WindowManager,
};
pub use tab::{
    TabId, ProcessId, RenderSurfaceId, Url, Tab, TabManager,
};
pub use message::{
    ShellMessage, ShellResponse, DownloadId, DownloadInfo, DownloadStatus,
    KeyboardShortcut, MenuAction, ComponentType,
};
