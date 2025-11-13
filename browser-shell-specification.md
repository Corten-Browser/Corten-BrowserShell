# Browser Shell Component Specification
## CortenBrowser Component v1.0

### Table of Contents
1. [Component Overview](#component-overview)
2. [Architecture](#architecture)
3. [External Interfaces](#external-interfaces)
4. [Internal Architecture](#internal-architecture)
5. [Implementation Phases](#implementation-phases)
6. [API Specifications](#api-specifications)
7. [Build Configuration](#build-configuration)
8. [Test Strategy](#test-strategy)
9. [Performance Requirements](#performance-requirements)
10. [Security Considerations](#security-considerations)
11. [Development Milestones](#development-milestones)

---

## Component Overview

### Purpose
The Browser Shell is the primary user interface and orchestration layer of CortenBrowser. It provides window management, tab management, browser chrome (UI elements), user input handling, and coordinates all other browser components through a central message bus.

### Responsibilities
- Window creation and management (multi-window support)
- Tab management and process isolation
- Browser chrome rendering (address bar, navigation buttons, menus)
- User input routing to appropriate components
- Component lifecycle management and orchestration
- Settings and preferences management
- Downloads management
- Bookmarks and history UI
- Extension UI integration points
- Developer tools hosting

### Current Implementation
- **Technology**: Tauri with WRY (WebView Rendering library)
- **Limitations**: Depends on system WebView, limited customization
- **Advantages**: Quick to implement, cross-platform support

### Target Implementation
- **Technology**: Pure Rust using egui or iced
- **Advantages**: Full control, consistent cross-platform behavior, better performance
- **Migration Strategy**: Gradual replacement maintaining API compatibility

### Component Statistics
- **Estimated Lines of Code**: 50,000-75,000
- **Development Time**: 3-4 weeks for Phase 1, 8-10 weeks for full implementation
- **Team Size**: Can be built by single Claude Code instance with proper partitioning

---

## Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         Browser Shell                            │
├───────────────┬─────────────┬─────────────┬────────────────────┤
│  UI Framework │ Window Mgr  │   Tab Mgr   │  Component Bus     │
│  (egui/iced)  │             │             │                    │
├───────────────┴─────────────┴─────────────┴────────────────────┤
│                     Platform Abstraction Layer                   │
├──────────────┬──────────────┬──────────────┬──────────────────┤
│    Linux     │   Windows    │    macOS     │    Web (WASM)    │
└──────────────┴──────────────┴──────────────┴──────────────────┘
```

### Component Dependencies

```rust
// External component dependencies
pub enum ComponentDependency {
    HtmlParser,      // For rendering UI HTML if using hybrid approach
    CssEngine,       // For styling browser chrome
    RenderEngine,    // For rendering web content
    NetworkStack,    // For loading resources
    DomImpl,         // For web page interaction
    JsRuntime,       // For extension scripts
    MediaEngine,     // For media controls
    FontSystem,      // For UI text rendering
    ExtensionSystem, // For extension UI integration
    DevTools,        // For developer tools UI
    Sandbox,         // For process isolation
    AdBlocker,       // For ad blocking UI
}
```

---

## External Interfaces

### Component Message Bus Interface

```rust
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[async_trait]
pub trait BrowserComponent: Send + Sync {
    /// Initialize component with configuration
    async fn initialize(&mut self, config: ComponentConfig) -> Result<(), ComponentError>;
    
    /// Shutdown component gracefully
    async fn shutdown(&mut self) -> Result<(), ComponentError>;
    
    /// Handle inter-component messages
    async fn handle_message(&mut self, msg: ComponentMessage) -> Result<ComponentResponse, ComponentError>;
    
    /// Get component health status
    fn health_check(&self) -> ComponentHealth;
    
    /// Get component metrics
    fn get_metrics(&self) -> ComponentMetrics;
}

/// Browser Shell specific messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShellMessage {
    // Window Management
    CreateWindow { id: WindowId, config: WindowConfig },
    CloseWindow { id: WindowId },
    ResizeWindow { id: WindowId, width: u32, height: u32 },
    MoveWindow { id: WindowId, x: i32, y: i32 },
    FocusWindow { id: WindowId },
    
    // Tab Management
    CreateTab { window_id: WindowId, url: Option<Url> },
    CloseTab { tab_id: TabId },
    NavigateTab { tab_id: TabId, url: Url },
    ReloadTab { tab_id: TabId, ignore_cache: bool },
    StopLoading { tab_id: TabId },
    GoBack { tab_id: TabId },
    GoForward { tab_id: TabId },
    
    // Browser Chrome
    UpdateAddressBar { tab_id: TabId, url: Url },
    UpdateTitle { tab_id: TabId, title: String },
    UpdateLoadingState { tab_id: TabId, loading: bool },
    ShowDownload { download_id: DownloadId, info: DownloadInfo },
    
    // User Input
    AddressBarInput { text: String },
    KeyboardShortcut { shortcut: KeyboardShortcut },
    MenuAction { action: MenuAction },
    
    // Component Coordination
    ComponentReady { component: ComponentType },
    ComponentError { component: ComponentType, error: String },
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
```

### Window Management API

```rust
/// Window management interface
pub trait WindowManager: Send + Sync {
    /// Create a new browser window
    async fn create_window(&mut self, config: WindowConfig) -> Result<WindowId, WindowError>;
    
    /// Close a window
    async fn close_window(&mut self, id: WindowId) -> Result<(), WindowError>;
    
    /// Get all windows
    fn get_windows(&self) -> Vec<&Window>;
    
    /// Get window by ID
    fn get_window(&self, id: WindowId) -> Option<&Window>;
    
    /// Update window properties
    async fn update_window(&mut self, id: WindowId, update: WindowUpdate) -> Result<(), WindowError>;
    
    /// Handle platform window events
    async fn handle_platform_event(&mut self, event: PlatformEvent) -> Result<(), WindowError>;
}

/// Window configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub fullscreen: bool,
    pub resizable: bool,
    pub decorations: bool,
    pub always_on_top: bool,
    pub skip_taskbar: bool,
}

/// Window representation
pub struct Window {
    pub id: WindowId,
    pub config: WindowConfig,
    pub tabs: Vec<TabId>,
    pub active_tab: Option<TabId>,
    pub platform_handle: PlatformWindowHandle,
}
```

### Tab Management API

```rust
/// Tab management interface
pub trait TabManager: Send + Sync {
    /// Create new tab
    async fn create_tab(&mut self, window_id: WindowId, url: Option<Url>) -> Result<TabId, TabError>;
    
    /// Close tab
    async fn close_tab(&mut self, tab_id: TabId) -> Result<(), TabError>;
    
    /// Navigate tab to URL
    async fn navigate(&mut self, tab_id: TabId, url: Url) -> Result<(), TabError>;
    
    /// Reload tab
    async fn reload(&mut self, tab_id: TabId, ignore_cache: bool) -> Result<(), TabError>;
    
    /// Stop loading
    async fn stop(&mut self, tab_id: TabId) -> Result<(), TabError>;
    
    /// Navigation history
    async fn go_back(&mut self, tab_id: TabId) -> Result<(), TabError>;
    async fn go_forward(&mut self, tab_id: TabId) -> Result<(), TabError>;
    
    /// Get tab information
    fn get_tab(&self, tab_id: TabId) -> Option<&Tab>;
    fn get_tabs(&self, window_id: WindowId) -> Vec<&Tab>;
    
    /// Switch active tab
    async fn activate_tab(&mut self, tab_id: TabId) -> Result<(), TabError>;
}

/// Tab representation
pub struct Tab {
    pub id: TabId,
    pub window_id: WindowId,
    pub title: String,
    pub url: Option<Url>,
    pub loading: bool,
    pub can_go_back: bool,
    pub can_go_forward: bool,
    pub favicon: Option<Vec<u8>>,
    pub process_id: Option<ProcessId>,
    pub render_surface: RenderSurfaceId,
}
```

---

## Internal Architecture

### Module Structure

```
browser-shell/
├── src/
│   ├── lib.rs                  # Component interface implementation
│   ├── main.rs                 # Standalone executable
│   ├── window/
│   │   ├── mod.rs              # Window management module
│   │   ├── manager.rs          # WindowManager implementation
│   │   ├── platform/
│   │   │   ├── linux.rs        # Linux-specific window code
│   │   │   ├── windows.rs      # Windows-specific window code
│   │   │   └── macos.rs        # macOS-specific window code
│   │   └── events.rs           # Window event handling
│   ├── tab/
│   │   ├── mod.rs              # Tab management module
│   │   ├── manager.rs          # TabManager implementation
│   │   ├── process.rs          # Tab process isolation
│   │   └── navigation.rs       # Navigation history
│   ├── ui/
│   │   ├── mod.rs              # UI framework integration
│   │   ├── chrome.rs           # Browser chrome components
│   │   ├── widgets/
│   │   │   ├── address_bar.rs  # Address bar widget
│   │   │   ├── tab_bar.rs      # Tab bar widget
│   │   │   ├── toolbar.rs      # Navigation toolbar
│   │   │   └── menu.rs         # Menu system
│   │   └── theme.rs            # Theming system
│   ├── bus/
│   │   ├── mod.rs              # Message bus implementation
│   │   ├── router.rs           # Message routing
│   │   ├── handler.rs          # Message handlers
│   │   └── async_bus.rs        # Async message processing
│   ├── settings/
│   │   ├── mod.rs              # Settings management
│   │   ├── preferences.rs      # User preferences
│   │   ├── storage.rs          # Settings persistence
│   │   └── sync.rs             # Settings sync
│   ├── downloads/
│   │   ├── mod.rs              # Downloads manager
│   │   ├── tracker.rs          # Download tracking
│   │   └── ui.rs               # Downloads UI
│   ├── bookmarks/
│   │   ├── mod.rs              # Bookmarks system
│   │   ├── storage.rs          # Bookmarks storage
│   │   └── ui.rs               # Bookmarks UI
│   └── platform/
│       ├── mod.rs              # Platform abstraction
│       ├── notifications.rs    # System notifications
│       └── clipboard.rs        # Clipboard integration
├── tests/
│   ├── unit/                   # Unit tests
│   ├── integration/            # Integration tests
│   └── ui/                     # UI tests
└── benches/
    └── performance.rs          # Performance benchmarks
```

### State Management

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

/// Main browser shell state
pub struct BrowserShell {
    // Window management
    windows: Arc<RwLock<HashMap<WindowId, Window>>>,
    window_manager: Arc<RwLock<Box<dyn WindowManager>>>,
    
    // Tab management
    tabs: Arc<RwLock<HashMap<TabId, Tab>>>,
    tab_manager: Arc<RwLock<Box<dyn TabManager>>>,
    
    // Component communication
    message_bus: Arc<MessageBus>,
    component_registry: Arc<RwLock<ComponentRegistry>>,
    
    // UI state
    ui_state: Arc<RwLock<UiState>>,
    theme: Arc<RwLock<Theme>>,
    
    // User data
    settings: Arc<RwLock<Settings>>,
    bookmarks: Arc<RwLock<BookmarkManager>>,
    downloads: Arc<RwLock<DownloadManager>>,
    history: Arc<RwLock<HistoryManager>>,
    
    // Platform integration
    platform: Arc<PlatformIntegration>,
}

/// UI state management
pub struct UiState {
    pub focused_window: Option<WindowId>,
    pub menu_visible: bool,
    pub fullscreen_mode: bool,
    pub developer_mode: bool,
    pub keyboard_shortcuts: HashMap<KeyboardShortcut, Action>,
}
```

### Threading Model

```rust
/// Browser shell runs on multiple threads
pub struct ThreadingModel {
    /// Main UI thread - handles all UI rendering and user input
    ui_thread: ThreadId,
    
    /// Message bus thread - routes messages between components
    bus_thread: ThreadId,
    
    /// IO thread pool - handles file I/O, settings, bookmarks
    io_pool: ThreadPool,
    
    /// Network thread - coordinates with network stack
    network_thread: ThreadId,
    
    /// Per-tab render threads - one per tab for isolation
    render_threads: HashMap<TabId, ThreadId>,
}

impl BrowserShell {
    /// Thread-safe message dispatch
    pub async fn dispatch_message(&self, msg: ComponentMessage) {
        // Messages are processed on appropriate thread
        match msg {
            ComponentMessage::UiUpdate(_) => {
                self.send_to_ui_thread(msg).await;
            }
            ComponentMessage::NetworkRequest(_) => {
                self.send_to_network_thread(msg).await;
            }
            ComponentMessage::RenderCommand(tab_id, _) => {
                self.send_to_render_thread(tab_id, msg).await;
            }
            _ => {
                self.message_bus.dispatch(msg).await;
            }
        }
    }
}
```

---

## Implementation Phases

### Phase 1: Tauri-based Shell (Week 1)
**Goal**: Minimal viable browser shell using Tauri/WRY

```rust
// Phase 1 implementation using Tauri
use tauri::{Builder, Manager, Window};
use wry::WebViewBuilder;

pub struct TauriBrowserShell {
    app: tauri::App,
    windows: Vec<Window>,
}

impl TauriBrowserShell {
    pub fn new() -> Result<Self, Error> {
        let app = Builder::default()
            .setup(|app| {
                let window = app.get_window("main").unwrap();
                // Configure WebView
                Ok(())
            })
            .build(tauri::generate_context!())?;
        
        Ok(Self {
            app,
            windows: vec![],
        })
    }
}
```

**Deliverables**:
- Basic window creation
- Tab management
- Address bar navigation
- Integration with network stack
- Basic message bus

### Phase 2: egui Migration (Weeks 2-3)
**Goal**: Replace Tauri UI with egui while maintaining WebView for content

```rust
// Phase 2 implementation using egui
use egui::{Context, CentralPanel, TopBottomPanel};
use eframe::{Frame, App};

pub struct EguiBrowserShell {
    tabs: Vec<Tab>,
    active_tab: usize,
    address_bar_text: String,
    webview: WebViewHandle,  // Still use WebView for content
}

impl App for EguiBrowserShell {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        // Top panel with address bar
        TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("←").clicked() {
                    self.go_back();
                }
                if ui.button("→").clicked() {
                    self.go_forward();
                }
                if ui.button("⟲").clicked() {
                    self.reload();
                }
                ui.text_edit_singleline(&mut self.address_bar_text);
                if ui.button("Go").clicked() {
                    self.navigate();
                }
            });
        });
        
        // Tab bar
        TopBottomPanel::top("tabs").show(ctx, |ui| {
            ui.horizontal(|ui| {
                for (i, tab) in self.tabs.iter().enumerate() {
                    if ui.selectable_label(i == self.active_tab, &tab.title).clicked() {
                        self.active_tab = i;
                    }
                }
                if ui.button("+").clicked() {
                    self.new_tab();
                }
            });
        });
        
        // Content area
        CentralPanel::default().show(ctx, |ui| {
            // Embed WebView here
            self.webview.render(ui);
        });
    }
}
```

**Deliverables**:
- egui-based browser chrome
- WebView embedding
- Full tab management
- Settings UI
- Downloads UI

### Phase 3: Pure Rust Shell (Weeks 4-6)
**Goal**: Complete Rust implementation with custom rendering

```rust
// Phase 3 full Rust implementation
pub struct PureRustBrowserShell {
    ui_framework: Box<dyn UiFramework>,
    render_engine: Arc<RenderEngine>,
    component_bus: Arc<MessageBus>,
    windows: Vec<BrowserWindow>,
}

impl PureRustBrowserShell {
    pub async fn create_window(&mut self) -> Result<WindowId, Error> {
        let window = BrowserWindow::new(
            self.ui_framework.create_window()?,
            self.render_engine.clone(),
            self.component_bus.clone(),
        );
        
        let window_id = window.id();
        self.windows.push(window);
        
        // Notify all components
        self.component_bus.broadcast(ComponentMessage::WindowCreated {
            id: window_id,
        }).await?;
        
        Ok(window_id)
    }
    
    pub async fn render_frame(&mut self, window_id: WindowId) {
        if let Some(window) = self.get_window_mut(window_id) {
            // Render browser chrome
            let chrome_commands = self.ui_framework.render_chrome(&window);
            
            // Get content from render engine
            let content_commands = self.render_engine.get_render_commands(
                window.active_tab()
            ).await;
            
            // Composite and display
            window.composite_and_display(chrome_commands, content_commands).await;
        }
    }
}
```

**Deliverables**:
- Complete Rust UI
- Direct render engine integration
- Full component orchestration
- Extension system integration
- Developer tools hosting

### Phase 4: Advanced Features (Weeks 7-8)
**Goal**: Production-ready features

**Features to implement**:
- Multi-window support
- Drag-and-drop tabs between windows
- Picture-in-picture
- PWA support
- Keyboard shortcuts customization
- Advanced downloads manager
- Password manager integration
- Sync system

---

## API Specifications

### Public Browser Shell API

```rust
/// Main Browser Shell API
pub struct BrowserShellAPI {
    shell: Arc<RwLock<BrowserShell>>,
}

impl BrowserShellAPI {
    /// Window operations
    pub async fn new_window(&self, config: WindowConfig) -> Result<WindowId> {
        self.shell.write().await.create_window(config).await
    }
    
    pub async fn close_window(&self, id: WindowId) -> Result<()> {
        self.shell.write().await.close_window(id).await
    }
    
    /// Tab operations
    pub async fn new_tab(&self, window_id: WindowId, url: Option<String>) -> Result<TabId> {
        self.shell.write().await.create_tab(window_id, url.map(Url::parse).transpose()?).await
    }
    
    pub async fn navigate(&self, tab_id: TabId, url: String) -> Result<()> {
        self.shell.write().await.navigate_tab(tab_id, Url::parse(&url)?).await
    }
    
    pub async fn reload(&self, tab_id: TabId) -> Result<()> {
        self.shell.write().await.reload_tab(tab_id, false).await
    }
    
    pub async fn go_back(&self, tab_id: TabId) -> Result<()> {
        self.shell.write().await.go_back(tab_id).await
    }
    
    pub async fn go_forward(&self, tab_id: TabId) -> Result<()> {
        self.shell.write().await.go_forward(tab_id).await
    }
    
    /// Bookmarks
    pub async fn add_bookmark(&self, tab_id: TabId) -> Result<BookmarkId> {
        let tab = self.shell.read().await.get_tab(tab_id)?;
        self.shell.write().await.add_bookmark(Bookmark {
            url: tab.url.clone(),
            title: tab.title.clone(),
            folder: None,
        }).await
    }
    
    pub async fn get_bookmarks(&self) -> Result<Vec<Bookmark>> {
        self.shell.read().await.get_all_bookmarks().await
    }
    
    /// Downloads
    pub async fn start_download(&self, url: String) -> Result<DownloadId> {
        self.shell.write().await.start_download(Url::parse(&url)?).await
    }
    
    pub async fn pause_download(&self, id: DownloadId) -> Result<()> {
        self.shell.write().await.pause_download(id).await
    }
    
    pub async fn resume_download(&self, id: DownloadId) -> Result<()> {
        self.shell.write().await.resume_download(id).await
    }
    
    /// Settings
    pub async fn get_setting(&self, key: &str) -> Result<SettingValue> {
        self.shell.read().await.get_setting(key).await
    }
    
    pub async fn set_setting(&self, key: &str, value: SettingValue) -> Result<()> {
        self.shell.write().await.set_setting(key, value).await
    }
}
```

### Extension Points API

```rust
/// API for extensions to interact with browser shell
pub trait ExtensionAPI: Send + Sync {
    /// Create browser action button
    async fn create_browser_action(&mut self, config: BrowserActionConfig) -> Result<ActionId>;
    
    /// Add context menu item
    async fn add_context_menu(&mut self, config: ContextMenuConfig) -> Result<MenuId>;
    
    /// Register keyboard shortcut
    async fn register_shortcut(&mut self, shortcut: KeyboardShortcut, action: ExtensionAction) -> Result<()>;
    
    /// Show notification
    async fn show_notification(&self, notification: Notification) -> Result<()>;
    
    /// Access tabs API
    fn tabs(&self) -> &dyn TabsAPI;
    
    /// Access windows API
    fn windows(&self) -> &dyn WindowsAPI;
    
    /// Access bookmarks API
    fn bookmarks(&self) -> &dyn BookmarksAPI;
}
```

---

## Build Configuration

### Cargo.toml

```toml
[package]
name = "browser-shell"
version = "0.1.0"
edition = "2021"
authors = ["CortenBrowser Team"]
license = "MIT OR Apache-2.0"

[dependencies]
# Core dependencies
tokio = { version = "1.35", features = ["full"] }
async-trait = "0.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"
anyhow = "1.0"

# Browser component interfaces
browser-interfaces = { path = "../shared/interfaces" }
browser-messages = { path = "../shared/messages" }
browser-types = { path = "../shared/types" }

# UI Framework (choose one)
egui = { version = "0.24", optional = true }
eframe = { version = "0.24", optional = true }
iced = { version = "0.10", optional = true }
tauri = { version = "1.5", optional = true, features = ["all"] }

# Platform specific
[target.'cfg(target_os = "linux")'.dependencies]
x11rb = "0.12"
wayland-client = "0.31"

[target.'cfg(target_os = "windows")'.dependencies]
windows = { version = "0.52", features = ["Win32_UI_WindowsAndMessaging", "Win32_Foundation"] }

[target.'cfg(target_os = "macos")'.dependencies]
cocoa = "0.25"
objc = "0.2"

# Networking and IPC
ipc-channel = "0.16"
bincode = "1.3"

# Storage
sqlite = "0.30"
directories = "5.0"

# Utilities
url = "2.5"
uuid = { version = "1.6", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
tracing = "0.1"
tracing-subscriber = "0.3"

# Testing
[dev-dependencies]
criterion = "0.5"
proptest = "1.4"
mockall = "0.12"
test-case = "3.1"

[features]
default = ["egui-ui"]
egui-ui = ["egui", "eframe"]
iced-ui = ["iced"]
tauri-ui = ["tauri"]
devtools = []
extensions = []

[profile.release]
opt-level = 3
lto = true
codegen-units = 1

[profile.dev]
opt-level = 0

[profile.test]
opt-level = 2

[[bench]]
name = "performance"
harness = false
```

### Build Script (build.rs)

```rust
use std::env;
use std::path::PathBuf;

fn main() {
    // Platform-specific build configuration
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    
    match target_os.as_str() {
        "linux" => {
            // Link with GTK/X11/Wayland libraries
            pkg_config::probe_library("gtk+-3.0").unwrap();
            println!("cargo:rustc-link-lib=X11");
        }
        "windows" => {
            // Windows specific setup
            println!("cargo:rustc-link-lib=user32");
            println!("cargo:rustc-link-lib=ole32");
        }
        "macos" => {
            // macOS frameworks
            println!("cargo:rustc-link-lib=framework=Cocoa");
            println!("cargo:rustc-link-lib=framework=WebKit");
        }
        _ => {}
    }
    
    // Generate component interface bindings
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    generate_component_bindings(&out_path);
}

fn generate_component_bindings(out_dir: &PathBuf) {
    // Generate Rust bindings for component interfaces
    // This would be expanded based on actual interface definitions
}
```

---

## Test Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_window_creation() {
        let shell = BrowserShell::new_test();
        let config = WindowConfig::default();
        let window_id = shell.create_window(config).await.unwrap();
        
        assert!(shell.get_window(window_id).is_some());
        assert_eq!(shell.windows.read().await.len(), 1);
    }
    
    #[tokio::test]
    async fn test_tab_navigation() {
        let shell = BrowserShell::new_test();
        let window_id = shell.create_window(WindowConfig::default()).await.unwrap();
        let tab_id = shell.create_tab(window_id, None).await.unwrap();
        
        let url = Url::parse("https://example.com").unwrap();
        shell.navigate_tab(tab_id, url.clone()).await.unwrap();
        
        let tab = shell.get_tab(tab_id).unwrap();
        assert_eq!(tab.url, Some(url));
    }
    
    #[test_case("https://google.com" ; "google")]
    #[test_case("https://github.com" ; "github")]
    #[test_case("file:///home/user/test.html" ; "local file")]
    async fn test_url_handling(url_str: &str) {
        let shell = BrowserShell::new_test();
        let url = Url::parse(url_str).unwrap();
        
        assert!(shell.validate_url(&url).is_ok());
    }
}
```

### Integration Tests

```rust
// tests/integration/component_communication.rs
#[tokio::test]
async fn test_component_message_routing() {
    let shell = create_test_shell().await;
    let mock_renderer = MockRenderer::new();
    
    shell.register_component(Box::new(mock_renderer)).await;
    
    // Send navigation message
    let response = shell.send_message(ComponentMessage::Navigate {
        tab_id: TabId::new(),
        url: Url::parse("https://example.com").unwrap(),
    }).await.unwrap();
    
    assert_matches!(response, ComponentResponse::NavigationStarted { .. });
}

#[tokio::test]
async fn test_multi_window_management() {
    let shell = create_test_shell().await;
    
    // Create multiple windows
    let window1 = shell.create_window(WindowConfig::default()).await.unwrap();
    let window2 = shell.create_window(WindowConfig::default()).await.unwrap();
    
    // Create tabs in each window
    let tab1 = shell.create_tab(window1, None).await.unwrap();
    let tab2 = shell.create_tab(window2, None).await.unwrap();
    
    // Test tab isolation
    assert_ne!(tab1, tab2);
    assert_eq!(shell.get_tab(tab1).unwrap().window_id, window1);
    assert_eq!(shell.get_tab(tab2).unwrap().window_id, window2);
}
```

### UI Tests

```rust
// tests/ui/address_bar_test.rs
#[test]
fn test_address_bar_input() {
    let app = create_test_app();
    let mut ui_test = UiTestHarness::new(app);
    
    // Type in address bar
    ui_test.type_text("address_bar", "https://rust-lang.org");
    ui_test.press_key(Key::Enter);
    
    // Verify navigation started
    assert_eq!(ui_test.get_current_url(), "https://rust-lang.org");
    assert!(ui_test.is_loading());
}

#[test]
fn test_tab_switching() {
    let app = create_test_app();
    let mut ui_test = UiTestHarness::new(app);
    
    // Create multiple tabs
    ui_test.click("new_tab_button");
    ui_test.click("new_tab_button");
    
    // Switch between tabs
    ui_test.click("tab_1");
    assert_eq!(ui_test.get_active_tab(), 1);
    
    ui_test.click("tab_0");
    assert_eq!(ui_test.get_active_tab(), 0);
}
```

### Performance Tests

```rust
// benches/performance.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_tab_creation(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let shell = runtime.block_on(create_test_shell());
    let window_id = runtime.block_on(shell.create_window(WindowConfig::default())).unwrap();
    
    c.bench_function("create_100_tabs", |b| {
        b.iter(|| {
            runtime.block_on(async {
                for _ in 0..100 {
                    black_box(shell.create_tab(window_id, None).await.unwrap());
                }
            });
        });
    });
}

fn bench_message_routing(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let shell = runtime.block_on(create_test_shell());
    
    c.bench_function("route_10k_messages", |b| {
        b.iter(|| {
            runtime.block_on(async {
                for _ in 0..10_000 {
                    shell.handle_message(ComponentMessage::Heartbeat).await;
                }
            });
        });
    });
}

criterion_group!(benches, bench_tab_creation, bench_message_routing);
criterion_main!(benches);
```

### Test Coverage Requirements

| Test Type | Coverage Target | Priority |
|-----------|----------------|----------|
| Unit Tests | 85% | High |
| Integration Tests | 70% | High |
| UI Tests | 60% | Medium |
| Performance Tests | Critical paths | High |

---

## Performance Requirements

### Latency Requirements

| Operation | Target | Maximum |
|-----------|--------|---------|
| Window creation | < 100ms | 200ms |
| Tab creation | < 50ms | 100ms |
| Tab switching | < 10ms | 20ms |
| URL navigation | < 5ms | 10ms |
| UI render frame | < 16ms | 33ms |
| Message routing | < 1ms | 5ms |

### Memory Requirements

| Component | Target | Maximum |
|-----------|--------|---------|
| Base shell process | < 50MB | 100MB |
| Per window overhead | < 10MB | 20MB |
| Per tab overhead | < 5MB | 10MB |
| UI framework | < 20MB | 40MB |

### Throughput Requirements

| Metric | Target |
|--------|--------|
| Messages/second | > 100,000 |
| Tabs supported | > 500 |
| Windows supported | > 50 |
| Bookmarks supported | > 10,000 |
| History entries | > 100,000 |

---

## Security Considerations

### Process Isolation

```rust
/// Tab process isolation configuration
pub struct ProcessIsolation {
    /// Each tab runs in separate process
    pub process_per_tab: bool,
    
    /// Maximum processes to create
    pub max_processes: usize,
    
    /// Process recycling after N navigations
    pub recycle_after: usize,
    
    /// Sandbox configuration
    pub sandbox_config: SandboxConfig,
}

impl Default for ProcessIsolation {
    fn default() -> Self {
        Self {
            process_per_tab: true,
            max_processes: 100,
            recycle_after: 5,
            sandbox_config: SandboxConfig::strict(),
        }
    }
}
```

### IPC Security

```rust
/// Secure IPC message validation
impl MessageValidator {
    pub fn validate(&self, msg: &ComponentMessage) -> Result<(), SecurityError> {
        // Validate message source
        self.validate_source(msg.source())?;
        
        // Check message size limits
        if msg.size() > MAX_MESSAGE_SIZE {
            return Err(SecurityError::MessageTooLarge);
        }
        
        // Validate message content
        self.validate_content(msg)?;
        
        // Check permissions
        self.check_permissions(msg)?;
        
        Ok(())
    }
}
```

### Input Sanitization

```rust
/// URL and input sanitization
impl InputSanitizer {
    pub fn sanitize_url(&self, url: &str) -> Result<Url, SanitizationError> {
        // Remove dangerous schemes
        if url.starts_with("javascript:") || url.starts_with("data:") {
            return Err(SanitizationError::DangerousScheme);
        }
        
        // Parse and validate
        let parsed = Url::parse(url)?;
        
        // Check against blacklist
        if self.is_blacklisted(&parsed) {
            return Err(SanitizationError::Blacklisted);
        }
        
        Ok(parsed)
    }
    
    pub fn sanitize_user_input(&self, input: &str) -> String {
        // Remove control characters
        input.chars()
            .filter(|c| !c.is_control())
            .collect()
    }
}
```

---

## Development Milestones

### Milestone 1: Basic Shell (Week 1)
**Deliverables**:
- [ ] Window creation and management
- [ ] Basic tab management
- [ ] Address bar navigation
- [ ] Message bus implementation
- [ ] Component registration

**Validation**:
- Create window with 3 tabs
- Navigate each tab to different URL
- Close tabs and window cleanly
- Message routing test suite passes

### Milestone 2: Browser Chrome (Week 2)
**Deliverables**:
- [ ] Complete toolbar implementation
- [ ] Tab bar with drag-and-drop
- [ ] Menu system
- [ ] Keyboard shortcuts
- [ ] Settings UI

**Validation**:
- All UI elements responsive
- Keyboard shortcuts functional
- Settings persist across restarts
- UI performance < 16ms per frame

### Milestone 3: Component Integration (Week 3)
**Deliverables**:
- [ ] Network stack integration
- [ ] Render engine integration
- [ ] HTML/CSS/DOM integration
- [ ] Ad blocker integration
- [ ] Extension system hooks

**Validation**:
- Load and render web pages
- Ad blocking functional
- Component communication stable
- 50% of integration tests pass

### Milestone 4: Advanced Features (Week 4)
**Deliverables**:
- [ ] Downloads manager
- [ ] Bookmarks system
- [ ] History tracking
- [ ] Find in page
- [ ] Print support

**Validation**:
- Download files successfully
- Bookmark management works
- History properly tracked
- Find highlights matches
- Print preview renders correctly

### Milestone 5: Platform Features (Week 5)
**Deliverables**:
- [ ] System notifications
- [ ] Clipboard integration
- [ ] Drag and drop support
- [ ] File type associations
- [ ] Protocol handlers

**Validation**:
- Notifications appear correctly
- Copy/paste works with web content
- Drag and drop files into browser
- Open HTML files from file manager
- Handle custom protocols

### Milestone 6: Production Ready (Week 6)
**Deliverables**:
- [ ] Performance optimization
- [ ] Memory leak fixes
- [ ] Security hardening
- [ ] Crash recovery
- [ ] Auto-update system

**Validation**:
- Pass all performance benchmarks
- No memory leaks in 24-hour test
- Security audit passed
- Graceful crash recovery
- Update system functional

---

## Appendix A: Message Protocol

### Component Message Format

```rust
/// Standard message format for all component communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentMessage {
    /// Unique message ID
    pub id: MessageId,
    
    /// Source component
    pub source: ComponentId,
    
    /// Target component(s)
    pub target: MessageTarget,
    
    /// Message timestamp
    pub timestamp: DateTime<Utc>,
    
    /// Message priority
    pub priority: MessagePriority,
    
    /// Message payload
    pub payload: MessagePayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageTarget {
    /// Send to specific component
    Component(ComponentId),
    
    /// Broadcast to all components
    Broadcast,
    
    /// Send to component group
    Group(ComponentGroup),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessagePriority {
    Critical,  // Must be processed immediately
    High,      // Process as soon as possible
    Normal,    // Regular processing
    Low,       // Process when idle
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessagePayload {
    // Navigation
    Navigate { tab_id: TabId, url: Url },
    NavigationComplete { tab_id: TabId, success: bool },
    
    // Rendering
    RenderFrame { tab_id: TabId, frame: RenderFrame },
    Paint { surface_id: SurfaceId, commands: Vec<PaintCommand> },
    
    // User input
    KeyPress { key: Key, modifiers: Modifiers },
    MouseClick { x: i32, y: i32, button: MouseButton },
    Scroll { dx: f32, dy: f32 },
    
    // Component lifecycle
    Initialize { config: ComponentConfig },
    Shutdown { graceful: bool },
    HealthCheck,
    
    // Data transfer
    LargeData { key: String, size: usize },  // Actual data in shared memory
}
```

## Appendix B: Configuration Schema

```toml
# browser-shell.toml - Configuration file

[window]
default_width = 1024
default_height = 768
min_width = 400
min_height = 300
allow_resize = true
start_maximized = false

[tabs]
enable_process_isolation = true
max_processes = 50
recycle_after_navigations = 5
restore_on_crash = true
lazy_loading = true

[ui]
theme = "light"  # light, dark, auto
show_bookmarks_bar = true
show_status_bar = false
animations_enabled = true
font_size = 14

[performance]
render_fps = 60
max_message_queue = 10000
compositor_threads = 4
raster_threads = 4

[security]
enable_sandbox = true
allow_javascript = true
allow_plugins = false
block_third_party_cookies = true
enable_webrtc = false

[network]
max_connections_per_host = 6
connection_timeout = 30
enable_http2 = true
enable_quic = true

[privacy]
do_not_track = true
clear_on_exit = false
private_browsing_available = true

[developer]
enable_devtools = true
enable_extensions = true
allow_experimental_features = false
```

## Appendix C: Platform-Specific Implementation

### Linux Implementation

```rust
// platform/linux.rs
use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;

pub struct LinuxWindow {
    connection: Arc<dyn Connection>,
    window_id: Window,
    screen: Screen,
}

impl LinuxWindow {
    pub fn create(config: &WindowConfig) -> Result<Self, Error> {
        let (conn, screen_num) = x11rb::connect(None)?;
        let screen = &conn.setup().roots[screen_num];
        
        let window_id = conn.generate_id()?;
        conn.create_window(
            COPY_DEPTH_FROM_PARENT,
            window_id,
            screen.root,
            config.x.unwrap_or(0) as i16,
            config.y.unwrap_or(0) as i16,
            config.width as u16,
            config.height as u16,
            0,
            WindowClass::INPUT_OUTPUT,
            screen.root_visual,
            &CreateWindowAux::new()
                .background_pixel(screen.white_pixel)
                .event_mask(EventMask::EXPOSURE | EventMask::STRUCTURE_NOTIFY),
        )?;
        
        Ok(Self {
            connection: Arc::new(conn),
            window_id,
            screen: screen.clone(),
        })
    }
}
```

### Windows Implementation

```rust
// platform/windows.rs
use windows::Win32::Foundation::*;
use windows::Win32::UI::WindowsAndMessaging::*;

pub struct WindowsWindow {
    hwnd: HWND,
    instance: HINSTANCE,
}

impl WindowsWindow {
    pub unsafe fn create(config: &WindowConfig) -> Result<Self, Error> {
        let instance = GetModuleHandleW(None)?;
        let class_name = w!("BrowserShellWindow");
        
        let wc = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(window_proc),
            hInstance: instance,
            lpszClassName: class_name,
            ..Default::default()
        };
        
        RegisterClassW(&wc);
        
        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            class_name,
            &config.title.to_wide(),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            config.x.unwrap_or(CW_USEDEFAULT),
            config.y.unwrap_or(CW_USEDEFAULT),
            config.width as i32,
            config.height as i32,
            None,
            None,
            instance,
            None,
        );
        
        Ok(Self { hwnd, instance })
    }
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    // Window message handling
    DefWindowProcW(hwnd, msg, wparam, lparam)
}
```

### macOS Implementation

```rust
// platform/macos.rs
use cocoa::base::{id, nil};
use cocoa::foundation::{NSAutoreleasePool, NSString};
use cocoa::appkit::{NSApp, NSApplication, NSWindow, NSWindowStyleMask};

pub struct MacWindow {
    window: id,
    pool: id,
}

impl MacWindow {
    pub unsafe fn create(config: &WindowConfig) -> Result<Self, Error> {
        let pool = NSAutoreleasePool::new(nil);
        
        let app = NSApp();
        app.setActivationPolicy_(
            cocoa::appkit::NSApplicationActivationPolicy::NSApplicationActivationPolicyRegular
        );
        
        let window = NSWindow::alloc(nil).initWithContentRect_styleMask_backing_defer_(
            NSRect::new(
                NSPoint::new(
                    config.x.unwrap_or(0) as f64,
                    config.y.unwrap_or(0) as f64
                ),
                NSSize::new(config.width as f64, config.height as f64),
            ),
            NSWindowStyleMask::NSTitledWindowMask
                | NSWindowStyleMask::NSClosableWindowMask
                | NSWindowStyleMask::NSResizableWindowMask
                | NSWindowStyleMask::NSMiniaturizableWindowMask,
            NSBackingStoreBuffered,
            NO,
        );
        
        let title = NSString::alloc(nil).init_str(&config.title);
        window.setTitle_(title);
        window.makeKeyAndOrderFront_(nil);
        
        Ok(Self { window, pool })
    }
}
```

---

## End of Specification

This specification provides comprehensive guidance for implementing the Browser Shell component. Each section can be used independently by Claude Code instances for specific implementation tasks. The specification will be updated as development progresses and requirements evolve.

**Version**: 1.0  
**Last Updated**: 2024  
**Total Implementation Estimate**: 50,000-75,000 lines of code  
**Development Timeline**: 6-8 weeks for full implementation
