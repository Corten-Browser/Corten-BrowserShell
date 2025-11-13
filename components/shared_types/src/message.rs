// @implements: REQ-006
//! Message Protocol
//!
//! This module defines the complete message protocol for browser shell communication.

use serde::{Deserialize, Serialize};
use crate::window::{WindowId, WindowConfig};
use crate::tab::{TabId, Url};

/// Browser Shell specific messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShellMessage {
    // Window Management
    CreateWindow {
        id: WindowId,
        config: WindowConfig,
    },
    CloseWindow {
        id: WindowId,
    },
    ResizeWindow {
        id: WindowId,
        width: u32,
        height: u32,
    },
    MoveWindow {
        id: WindowId,
        x: i32,
        y: i32,
    },
    FocusWindow {
        id: WindowId,
    },

    // Tab Management
    CreateTab {
        window_id: WindowId,
        url: Option<Url>,
    },
    CloseTab {
        tab_id: TabId,
    },
    NavigateTab {
        tab_id: TabId,
        url: Url,
    },
    ReloadTab {
        tab_id: TabId,
        ignore_cache: bool,
    },
    StopLoading {
        tab_id: TabId,
    },
    GoBack {
        tab_id: TabId,
    },
    GoForward {
        tab_id: TabId,
    },

    // Browser Chrome Updates
    UpdateAddressBar {
        tab_id: TabId,
        url: Url,
    },
    UpdateTitle {
        tab_id: TabId,
        title: String,
    },
    UpdateLoadingState {
        tab_id: TabId,
        loading: bool,
    },
    ShowDownload {
        download_id: DownloadId,
        info: DownloadInfo,
    },

    // User Input
    AddressBarInput {
        text: String,
    },
    KeyboardShortcut {
        shortcut: KeyboardShortcut,
    },
    MenuAction {
        action: MenuAction,
    },

    // Component Coordination
    ComponentReady {
        component: ComponentType,
    },
    ComponentError {
        component: ComponentType,
        error: String,
    },
    RequestShutdown,
}

/// Shell responses to other components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShellResponse {
    WindowCreated { id: WindowId },
    TabCreated { id: TabId },
    NavigationStarted { tab_id: TabId },
    ShutdownAcknowledged,
    Error { message: String },
}

/// Download identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DownloadId(pub u64);

/// Download information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadInfo {
    pub url: Url,
    pub filename: String,
    pub total_bytes: Option<u64>,
    pub downloaded_bytes: u64,
    pub status: DownloadStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DownloadStatus {
    Pending,
    InProgress,
    Paused,
    Completed,
    Failed { reason: String },
    Cancelled,
}

/// Keyboard shortcut definition
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyboardShortcut {
    pub key: String,
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool,
}

/// Menu actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MenuAction {
    NewWindow,
    NewTab,
    CloseTab,
    CloseWindow,
    Reload,
    ReloadIgnoreCache,
    Stop,
    Back,
    Forward,
    ZoomIn,
    ZoomOut,
    ZoomReset,
    ToggleFullscreen,
    ShowSettings,
    ShowBookmarks,
    ShowHistory,
    ShowDownloads,
    ShowDevTools,
    Quit,
}

/// Component type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComponentType {
    HtmlParser,
    CssEngine,
    RenderEngine,
    NetworkStack,
    DomImpl,
    JsRuntime,
    MediaEngine,
    FontSystem,
    ExtensionSystem,
    DevTools,
    Sandbox,
    AdBlocker,
    BrowserShell,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_message_create_window() {
        let msg = ShellMessage::CreateWindow {
            id: WindowId(123),
            config: WindowConfig::default(),
        };
        match msg {
            ShellMessage::CreateWindow { id, .. } => {
                assert_eq!(id.0, 123);
            }
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn shell_response_window_created() {
        let resp = ShellResponse::WindowCreated { id: WindowId(456) };
        match resp {
            ShellResponse::WindowCreated { id } => {
                assert_eq!(id.0, 456);
            }
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn download_status_variants() {
        let pending = DownloadStatus::Pending;
        let failed = DownloadStatus::Failed {
            reason: "network error".to_string()
        };

        match pending {
            DownloadStatus::Pending => {},
            _ => panic!("Wrong variant"),
        }

        match failed {
            DownloadStatus::Failed { reason } => {
                assert_eq!(reason, "network error");
            }
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn keyboard_shortcut_creation() {
        let shortcut = KeyboardShortcut {
            key: "T".to_string(),
            ctrl: true,
            alt: false,
            shift: false,
            meta: false,
        };
        assert_eq!(shortcut.key, "T");
        assert!(shortcut.ctrl);
        assert!(!shortcut.alt);
    }

    #[test]
    fn component_type_enum() {
        let comp = ComponentType::RenderEngine;
        assert_eq!(comp, ComponentType::RenderEngine);
        assert_ne!(comp, ComponentType::NetworkStack);
    }

    #[test]
    fn menu_action_variants() {
        let action = MenuAction::NewTab;
        match action {
            MenuAction::NewTab => {},
            _ => panic!("Wrong variant"),
        }
    }
}
