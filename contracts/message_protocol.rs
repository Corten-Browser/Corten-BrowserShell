// Message Protocol Contract
// Version: 0.17.0
//
// This contract defines the complete message protocol for browser shell communication.

use serde::{Deserialize, Serialize};
use super::window_manager::{WindowId, WindowConfig};
use super::tab_manager::{TabId, Url};

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
