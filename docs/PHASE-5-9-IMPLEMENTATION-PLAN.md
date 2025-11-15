# Browser Shell Completion Plan - Phases 5-9

**Project**: CortenBrowser Browser Shell
**Current Version**: v0.4.0
**Target Version**: v1.0.0
**Plan Created**: 2025-11-14
**Estimated Duration**: 5-6 weeks (autonomous implementation)

---

## Executive Summary

This plan completes the browser shell specification by implementing all missing milestones:
- **Phase 5**: WebView Integration & UI Completeness (v0.5.0)
- **Phase 6**: Content Features UI (v0.6.0)
- **Phase 7**: Platform Integration (v0.7.0)
- **Phase 8**: Extension & DevTools Hosting (v0.8.0)
- **Phase 9**: Production Hardening (v0.9.0 → v1.0.0)

**Current State**: v0.4.0 (~50% of shell spec complete)
**Target State**: v1.0.0 (100% of shell spec complete)

---

## Phase 5: WebView Integration & UI Completeness (v0.5.0)

**Goal**: Integrate web content rendering and complete browser chrome
**Duration**: Week 1-2
**Dependencies**: None (can start immediately)

### 5.1: WebView Integration (CRITICAL)

**Component**: `content_area` (new)

**Implementation**:
```rust
// components/content_area/src/lib.rs
use wry::WebView;
use egui::Ui;

pub struct ContentArea {
    webview: Option<WebView>,
    current_url: Option<String>,
}

impl ContentArea {
    pub fn new() -> Self { /* ... */ }

    pub fn navigate(&mut self, url: &str) -> Result<()> {
        // Navigate webview to URL
    }

    pub fn render(&mut self, ui: &mut Ui) {
        // Embed webview in egui UI
    }
}
```

**Integration Points**:
- Coordinate with `window_manager` for window handles
- Receive navigation commands from `ui_chrome`
- Send loading status to `tab_manager`

**Testing**:
- Load basic HTML page
- Navigate between URLs
- Handle navigation errors
- Verify webview displays in content area

**Deliverables**:
- `components/content_area/` - new component
- Integration with `ui_chrome` for navigation
- Tests: 20+ (webview creation, navigation, error handling)

---

### 5.2: Full Menu System

**Component**: `ui_chrome` (enhancement)

**Implementation**:
```rust
// Add to ui_chrome/src/lib.rs
pub struct MenuBar {
    file_menu: FileMenu,
    edit_menu: EditMenu,
    view_menu: ViewMenu,
    history_menu: HistoryMenu,
    bookmarks_menu: BookmarksMenu,
    tools_menu: ToolsMenu,
    help_menu: HelpMenu,
}

impl MenuBar {
    pub fn render(&mut self, ui: &mut egui::Ui) {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("New Window").clicked() { /* ... */ }
                if ui.button("New Tab").clicked() { /* ... */ }
                ui.separator();
                if ui.button("Open File...").clicked() { /* ... */ }
                ui.separator();
                if ui.button("Exit").clicked() { /* ... */ }
            });

            ui.menu_button("Edit", |ui| {
                if ui.button("Cut").clicked() { /* ... */ }
                if ui.button("Copy").clicked() { /* ... */ }
                if ui.button("Paste").clicked() { /* ... */ }
                ui.separator();
                if ui.button("Find...").clicked() { /* ... */ }
            });

            // View, History, Bookmarks, Tools, Help...
        });
    }
}
```

**Menus to Implement**:
1. **File**: New Window, New Tab, Open File, Close Tab, Exit
2. **Edit**: Cut, Copy, Paste, Find, Preferences
3. **View**: Zoom In/Out/Reset, Fullscreen, Developer Tools
4. **History**: Back, Forward, Recent, Show All
5. **Bookmarks**: Add Bookmark, Show All, Organize
6. **Tools**: Downloads, Extensions, Task Manager
7. **Help**: About, Documentation, Report Issue

**Testing**:
- All menu items clickable
- Menu actions trigger correct events
- Keyboard shortcuts work from menus
- Tests: 30+ (one per menu item)

**Deliverables**:
- Complete menu bar in `ui_chrome`
- Menu action routing via message bus
- Tests: 30+ menu interaction tests

---

### 5.3: Tab Drag-and-Drop

**Component**: `ui_chrome` (enhancement to tab bar)

**Implementation**:
```rust
// Add to ui_chrome/src/widgets/tab_bar.rs
pub struct TabBar {
    tabs: Vec<TabWidget>,
    drag_state: Option<TabDragState>,
}

struct TabDragState {
    tab_id: TabId,
    start_pos: egui::Pos2,
    current_pos: egui::Pos2,
    dragging_to_window: Option<WindowId>,
}

impl TabBar {
    pub fn render(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            for (i, tab) in self.tabs.iter_mut().enumerate() {
                let response = ui.selectable_label(
                    tab.is_active,
                    &tab.title
                );

                // Handle drag start
                if response.drag_started() {
                    self.drag_state = Some(TabDragState {
                        tab_id: tab.id,
                        start_pos: response.interact_pointer_pos().unwrap(),
                        current_pos: response.interact_pointer_pos().unwrap(),
                        dragging_to_window: None,
                    });
                }

                // Handle drag update
                if let Some(drag) = &mut self.drag_state {
                    if response.dragged() {
                        drag.current_pos = response.interact_pointer_pos().unwrap();

                        // Reorder tabs if dragging within same window
                        // Or create new window if dragged far enough
                    }
                }

                // Handle drag end
                if response.drag_released() {
                    if let Some(drag) = self.drag_state.take() {
                        // Finalize tab move/reorder
                    }
                }
            }
        });
    }
}
```

**Features**:
- Drag tabs to reorder within window
- Drag tab out to create new window
- Drag tab to different window
- Visual feedback during drag

**Testing**:
- Reorder tabs within window
- Drag tab to create new window
- Drag tab between windows
- Cancel drag (ESC key)
- Tests: 15+

**Deliverables**:
- Tab drag-and-drop in `ui_chrome`
- Multi-window tab transfer
- Tests: 15+ drag-and-drop tests

---

### Phase 5 Summary

**Version**: v0.4.0 → v0.5.0

**Deliverables**:
- ✅ WebView integration (content area component)
- ✅ Full menu system (7 menus)
- ✅ Tab drag-and-drop

**Tests Added**: ~65 tests
**Components Modified**: `ui_chrome`, `window_manager`, `tab_manager`
**Components Created**: `content_area`

**Milestone**: Browser can now display web pages!

---

## Phase 6: Content Features UI (v0.6.0)

**Goal**: Add history, find, and print UI
**Duration**: Week 2-3
**Dependencies**: Phase 5 complete (WebView needed for find/print)

### 6.1: History UI Panel

**Component**: `history_manager` (new)

**Implementation**:
```rust
// components/history_manager/src/lib.rs
use chrono::{DateTime, Utc};

#[derive(Clone, Debug)]
pub struct HistoryEntry {
    pub id: HistoryId,
    pub url: String,
    pub title: String,
    pub visit_time: DateTime<Utc>,
    pub visit_count: u32,
}

pub struct HistoryManager {
    entries: Vec<HistoryEntry>,
    db: HistoryDatabase,
}

impl HistoryManager {
    pub fn add_visit(&mut self, url: String, title: String) -> Result<()> {
        // Add to database
    }

    pub fn search(&self, query: &str) -> Vec<HistoryEntry> {
        // Search by URL or title
    }

    pub fn get_recent(&self, limit: usize) -> Vec<HistoryEntry> {
        // Get recent history
    }

    pub fn delete_entry(&mut self, id: HistoryId) -> Result<()> {
        // Delete history entry
    }

    pub fn clear_all(&mut self) -> Result<()> {
        // Clear all history
    }
}

// UI component in ui_chrome
pub struct HistoryPanel {
    manager: Arc<RwLock<HistoryManager>>,
    search_query: String,
    entries: Vec<HistoryEntry>,
}

impl HistoryPanel {
    pub fn render(&mut self, ui: &mut egui::Ui) {
        ui.heading("History");

        // Search box
        ui.text_edit_singleline(&mut self.search_query);

        // History list
        egui::ScrollArea::vertical().show(ui, |ui| {
            for entry in &self.entries {
                ui.horizontal(|ui| {
                    if ui.link(&entry.title).clicked() {
                        // Navigate to URL
                    }
                    ui.label(&entry.url);
                    ui.label(format!("{}", entry.visit_time.format("%Y-%m-%d %H:%M")));
                });
            }
        });

        // Clear history button
        if ui.button("Clear All History").clicked() {
            self.manager.write().unwrap().clear_all().ok();
        }
    }
}
```

**Database**: SQLite for history storage

**Features**:
- Track visited URLs with timestamps
- Search history by URL or title
- View recent history
- Delete individual entries
- Clear all history
- History panel in UI

**Testing**:
- Add history entries
- Search history
- Delete entries
- Clear all
- Tests: 20+

**Deliverables**:
- `components/history_manager/` - new component
- History panel in `ui_chrome`
- SQLite database for persistence
- Tests: 20+

---

### 6.2: Find in Page UI

**Component**: `ui_chrome` (find bar widget)

**Implementation**:
```rust
// components/ui_chrome/src/widgets/find_bar.rs
pub struct FindBar {
    search_query: String,
    case_sensitive: bool,
    match_count: usize,
    current_match: usize,
    visible: bool,
}

impl FindBar {
    pub fn render(&mut self, ui: &mut egui::Ui) -> Option<FindAction> {
        if !self.visible {
            return None;
        }

        let mut action = None;

        ui.horizontal(|ui| {
            ui.label("Find:");

            let response = ui.text_edit_singleline(&mut self.search_query);

            // Search as user types
            if response.changed() {
                action = Some(FindAction::Search(self.search_query.clone()));
            }

            // Previous match
            if ui.button("◀").clicked() {
                action = Some(FindAction::Previous);
            }

            // Next match
            if ui.button("▶").clicked() {
                action = Some(FindAction::Next);
            }

            // Match count
            ui.label(format!("{}/{}", self.current_match, self.match_count));

            // Case sensitive checkbox
            ui.checkbox(&mut self.case_sensitive, "Match case");

            // Close button
            if ui.button("✖").clicked() {
                self.visible = false;
                action = Some(FindAction::Close);
            }
        });

        action
    }

    pub fn show(&mut self) {
        self.visible = true;
    }
}

pub enum FindAction {
    Search(String),
    Next,
    Previous,
    Close,
}
```

**Integration**:
- Send find requests to WebView
- Receive match count from WebView
- Highlight matches in page
- Navigate between matches

**Keyboard Shortcuts**:
- Ctrl+F: Open find bar
- Enter: Next match
- Shift+Enter: Previous match
- Esc: Close find bar

**Testing**:
- Find text in page
- Navigate between matches
- Case-sensitive search
- Close find bar
- Tests: 15+

**Deliverables**:
- Find bar widget in `ui_chrome`
- WebView integration for find
- Tests: 15+

---

### 6.3: Print UI

**Component**: `ui_chrome` (print dialog)

**Implementation**:
```rust
// components/ui_chrome/src/dialogs/print_dialog.rs
pub struct PrintDialog {
    visible: bool,
    preview: Option<PrintPreview>,
    settings: PrintSettings,
}

pub struct PrintSettings {
    pub printer: String,
    pub pages: PageRange,
    pub copies: u32,
    pub color: bool,
    pub orientation: Orientation,
    pub paper_size: PaperSize,
}

impl PrintDialog {
    pub fn render(&mut self, ui: &mut egui::Ui) -> Option<PrintAction> {
        if !self.visible {
            return None;
        }

        egui::Window::new("Print")
            .collapsible(false)
            .show(ui.ctx(), |ui| {
                // Printer selection
                egui::ComboBox::from_label("Printer")
                    .selected_text(&self.settings.printer)
                    .show_ui(ui, |ui| {
                        // List available printers
                    });

                // Page range
                ui.horizontal(|ui| {
                    ui.label("Pages:");
                    ui.radio_value(&mut self.settings.pages, PageRange::All, "All");
                    ui.radio_value(&mut self.settings.pages, PageRange::Current, "Current");
                    ui.radio_value(&mut self.settings.pages, PageRange::Custom, "Custom");
                });

                // Copies
                ui.horizontal(|ui| {
                    ui.label("Copies:");
                    ui.add(egui::DragValue::new(&mut self.settings.copies).speed(1));
                });

                // Color
                ui.checkbox(&mut self.settings.color, "Print in color");

                // Orientation
                ui.horizontal(|ui| {
                    ui.label("Orientation:");
                    ui.radio_value(&mut self.settings.orientation, Orientation::Portrait, "Portrait");
                    ui.radio_value(&mut self.settings.orientation, Orientation::Landscape, "Landscape");
                });

                // Print preview
                if let Some(preview) = &self.preview {
                    ui.separator();
                    ui.label("Preview:");
                    // Render preview
                }

                // Buttons
                ui.horizontal(|ui| {
                    if ui.button("Print").clicked() {
                        return Some(PrintAction::Print(self.settings.clone()));
                    }
                    if ui.button("Cancel").clicked() {
                        self.visible = false;
                        return Some(PrintAction::Cancel);
                    }
                });

                None
            })
            .and_then(|inner| inner.inner.flatten())
    }

    pub fn show(&mut self) {
        self.visible = true;
    }
}

pub enum PrintAction {
    Print(PrintSettings),
    Cancel,
}
```

**Features**:
- Printer selection
- Page range (all, current, custom)
- Number of copies
- Color/grayscale
- Orientation (portrait/landscape)
- Paper size
- Print preview

**Integration**:
- Get page content from WebView
- Generate print preview
- Send to OS print system

**Testing**:
- Open print dialog
- Change settings
- Generate preview
- Cancel print
- Tests: 12+

**Deliverables**:
- Print dialog in `ui_chrome`
- Print preview generation
- OS print integration
- Tests: 12+

---

### Phase 6 Summary

**Version**: v0.5.0 → v0.6.0

**Deliverables**:
- ✅ History tracking and UI
- ✅ Find in page
- ✅ Print dialog and preview

**Tests Added**: ~47 tests
**Components Created**: `history_manager`
**Components Modified**: `ui_chrome`

---

## Phase 7: Platform Integration (v0.7.0)

**Goal**: OS-level integration features
**Duration**: Week 3-4
**Dependencies**: Phase 5 complete (WebView needed)

### 7.1: System Notifications

**Component**: `platform_abstraction` (enhancement)

**Implementation**:
```rust
// components/platform_abstraction/src/notifications.rs
#[cfg(target_os = "linux")]
use notify_rust::Notification;

#[cfg(target_os = "windows")]
use winrt_notification::Toast;

#[cfg(target_os = "macos")]
use mac_notification_sys::Notification as MacNotification;

pub struct NotificationManager {
    #[cfg(target_os = "linux")]
    linux_impl: LinuxNotifications,

    #[cfg(target_os = "windows")]
    windows_impl: WindowsNotifications,

    #[cfg(target_os = "macos")]
    macos_impl: MacOSNotifications,
}

impl NotificationManager {
    pub fn show(&self, notification: BrowserNotification) -> Result<()> {
        #[cfg(target_os = "linux")]
        self.linux_impl.show(notification)?;

        #[cfg(target_os = "windows")]
        self.windows_impl.show(notification)?;

        #[cfg(target_os = "macos")]
        self.macos_impl.show(notification)?;

        Ok(())
    }
}

pub struct BrowserNotification {
    pub title: String,
    pub body: String,
    pub icon: Option<Vec<u8>>,
    pub actions: Vec<NotificationAction>,
}
```

**Use Cases**:
- Download complete
- Extension notifications
- Web notifications from pages
- Update available

**Testing**:
- Show notification
- Click notification action
- Close notification
- Tests: 10+

**Deliverables**:
- Notification system in `platform_abstraction`
- Integration with downloads, extensions
- Tests: 10+

---

### 7.2: Clipboard Integration

**Component**: `platform_abstraction` (enhancement)

**Implementation**:
```rust
// components/platform_abstraction/src/clipboard.rs
use arboard::Clipboard;

pub struct ClipboardManager {
    clipboard: Clipboard,
}

impl ClipboardManager {
    pub fn copy_text(&mut self, text: &str) -> Result<()> {
        self.clipboard.set_text(text)?;
        Ok(())
    }

    pub fn paste_text(&mut self) -> Result<String> {
        self.clipboard.get_text()
    }

    pub fn copy_image(&mut self, image: &[u8]) -> Result<()> {
        // Copy image to clipboard
    }

    pub fn paste_image(&mut self) -> Result<Vec<u8>> {
        // Paste image from clipboard
    }
}
```

**Integration**:
- Edit menu (Cut, Copy, Paste)
- WebView copy/paste coordination
- Context menu copy/paste

**Testing**:
- Copy text
- Paste text
- Copy image
- Paste image
- Tests: 10+

**Deliverables**:
- Clipboard manager in `platform_abstraction`
- Menu integration
- Tests: 10+

---

### 7.3: Drag-and-Drop Support

**Component**: `ui_chrome` (drag-and-drop handler)

**Implementation**:
```rust
// components/ui_chrome/src/drag_drop.rs
pub struct DragDropHandler {
    dragging: bool,
    drag_data: Option<DragData>,
}

pub enum DragData {
    Files(Vec<PathBuf>),
    Text(String),
    Url(String),
}

impl DragDropHandler {
    pub fn handle_drag_enter(&mut self, data: DragData) {
        self.dragging = true;
        self.drag_data = Some(data);
    }

    pub fn handle_drop(&mut self, pos: egui::Pos2) -> Option<DropAction> {
        if let Some(data) = self.drag_data.take() {
            self.dragging = false;

            match data {
                DragData::Files(files) => {
                    // Open files in new tabs
                    Some(DropAction::OpenFiles(files))
                }
                DragData::Text(text) => {
                    // Search for text or navigate if URL
                    Some(DropAction::HandleText(text))
                }
                DragData::Url(url) => {
                    // Navigate to URL
                    Some(DropAction::Navigate(url))
                }
            }
        } else {
            None
        }
    }
}

pub enum DropAction {
    OpenFiles(Vec<PathBuf>),
    HandleText(String),
    Navigate(String),
}
```

**Features**:
- Drag files into browser to open
- Drag URLs to navigate
- Drag text to search
- Visual drop indicator

**Testing**:
- Drag HTML file to browser
- Drag URL to browser
- Drag text to browser
- Tests: 10+

**Deliverables**:
- Drag-and-drop handler in `ui_chrome`
- File opening integration
- Tests: 10+

---

### 7.4: File Type Associations

**Component**: `platform_abstraction` (file associations)

**Implementation**:
```rust
// components/platform_abstraction/src/file_associations.rs
pub struct FileAssociationManager {
    registered_types: Vec<String>,
}

impl FileAssociationManager {
    pub fn register_as_default_browser(&self) -> Result<()> {
        #[cfg(target_os = "linux")]
        self.linux_register()?;

        #[cfg(target_os = "windows")]
        self.windows_register()?;

        #[cfg(target_os = "macos")]
        self.macos_register()?;

        Ok(())
    }

    pub fn register_file_type(&mut self, extension: &str) -> Result<()> {
        // Register .html, .htm, .xhtml, etc.
    }

    pub fn is_default_browser(&self) -> bool {
        // Check if this browser is the default
    }
}
```

**File Types**:
- .html, .htm
- .xhtml
- .svg
- .pdf (optional)

**Testing**:
- Register as default
- Check default status
- Open file from file manager
- Tests: 8+

**Deliverables**:
- File association system in `platform_abstraction`
- Registration UI in settings
- Tests: 8+

---

### 7.5: Protocol Handlers

**Component**: `platform_abstraction` (protocol handlers)

**Implementation**:
```rust
// components/platform_abstraction/src/protocols.rs
pub struct ProtocolHandler {
    registered_protocols: HashMap<String, ProtocolAction>,
}

pub enum ProtocolAction {
    OpenInBrowser,
    OpenExternal(String), // External app
    Custom(Box<dyn Fn(&str) -> Result<()>>),
}

impl ProtocolHandler {
    pub fn register_protocol(&mut self, protocol: &str, action: ProtocolAction) -> Result<()> {
        self.registered_protocols.insert(protocol.to_string(), action);

        // Register with OS
        #[cfg(target_os = "linux")]
        self.linux_register_protocol(protocol)?;

        #[cfg(target_os = "windows")]
        self.windows_register_protocol(protocol)?;

        #[cfg(target_os = "macos")]
        self.macos_register_protocol(protocol)?;

        Ok(())
    }

    pub fn handle(&self, url: &str) -> Result<()> {
        let protocol = url.split(':').next().unwrap_or("");

        if let Some(action) = self.registered_protocols.get(protocol) {
            match action {
                ProtocolAction::OpenInBrowser => {
                    // Open URL in browser
                }
                ProtocolAction::OpenExternal(app) => {
                    // Launch external app
                }
                ProtocolAction::Custom(handler) => {
                    handler(url)?;
                }
            }
        }

        Ok(())
    }
}
```

**Protocols**:
- http://, https:// (already handled)
- mailto: (open email client)
- magnet: (torrent handler)
- Custom protocols for extensions

**Testing**:
- Register protocol
- Handle mailto: link
- Handle magnet: link
- Tests: 10+

**Deliverables**:
- Protocol handler system in `platform_abstraction`
- mailto: and magnet: support
- Tests: 10+

---

### Phase 7 Summary

**Version**: v0.6.0 → v0.7.0

**Deliverables**:
- ✅ System notifications
- ✅ Clipboard integration
- ✅ Drag-and-drop support
- ✅ File type associations
- ✅ Protocol handlers

**Tests Added**: ~48 tests
**Components Modified**: `platform_abstraction`, `ui_chrome`

---

## Phase 8: Extension & DevTools Hosting (v0.8.0)

**Goal**: Provide UI integration points for extensions and developer tools
**Duration**: Week 4-5
**Dependencies**: Phase 5 complete (WebView needed)

### 8.1: Extension UI Hosting

**Component**: `extension_host` (new)

**Implementation**:
```rust
// components/extension_host/src/lib.rs
pub struct ExtensionHost {
    extensions: HashMap<ExtensionId, Extension>,
    browser_actions: Vec<BrowserAction>,
    context_menu_items: Vec<ContextMenuItem>,
}

pub struct BrowserAction {
    pub extension_id: ExtensionId,
    pub icon: Vec<u8>,
    pub title: String,
    pub popup: Option<String>, // HTML for popup
    pub badge_text: Option<String>,
    pub badge_color: Color,
}

pub struct ContextMenuItem {
    pub extension_id: ExtensionId,
    pub title: String,
    pub contexts: Vec<ContextType>,
    pub onclick: String, // Script to run
}

impl ExtensionHost {
    pub fn add_browser_action(&mut self, action: BrowserAction) -> Result<()> {
        self.browser_actions.push(action);
        Ok(())
    }

    pub fn add_context_menu_item(&mut self, item: ContextMenuItem) -> Result<()> {
        self.context_menu_items.push(item);
        Ok(())
    }

    pub fn render_toolbar_icons(&self, ui: &mut egui::Ui) {
        for action in &self.browser_actions {
            if ui.image_button(/* icon */).clicked() {
                // Show popup or trigger action
            }
        }
    }
}
```

**UI Integration Points**:
- Browser action icons in toolbar
- Extension context menu items
- Extension panels in sidebar
- Extension popups

**Testing**:
- Add browser action
- Click browser action
- Show extension popup
- Add context menu item
- Tests: 15+

**Deliverables**:
- `components/extension_host/` - new component
- UI integration in `ui_chrome`
- Tests: 15+

---

### 8.2: DevTools Hosting

**Component**: `devtools_host` (new)

**Implementation**:
```rust
// components/devtools_host/src/lib.rs
pub struct DevToolsHost {
    visible: bool,
    selected_panel: DevToolsPanel,
    docked: DockPosition,
}

pub enum DevToolsPanel {
    Elements,
    Console,
    Network,
    Performance,
    Application,
}

pub enum DockPosition {
    Bottom,
    Right,
    Detached,
}

impl DevToolsHost {
    pub fn render(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        if !self.visible {
            return;
        }

        match self.docked {
            DockPosition::Bottom => {
                egui::TopBottomPanel::bottom("devtools")
                    .resizable(true)
                    .show(ctx, |ui| {
                        self.render_panels(ui);
                    });
            }
            DockPosition::Right => {
                egui::SidePanel::right("devtools")
                    .resizable(true)
                    .show(ctx, |ui| {
                        self.render_panels(ui);
                    });
            }
            DockPosition::Detached => {
                egui::Window::new("Developer Tools")
                    .show(ctx, |ui| {
                        self.render_panels(ui);
                    });
            }
        }
    }

    fn render_panels(&mut self, ui: &mut egui::Ui) {
        // Panel tabs
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.selected_panel, DevToolsPanel::Elements, "Elements");
            ui.selectable_value(&mut self.selected_panel, DevToolsPanel::Console, "Console");
            ui.selectable_value(&mut self.selected_panel, DevToolsPanel::Network, "Network");
            ui.selectable_value(&mut self.selected_panel, DevToolsPanel::Performance, "Performance");
        });

        ui.separator();

        // Panel content
        match self.selected_panel {
            DevToolsPanel::Elements => self.render_elements_panel(ui),
            DevToolsPanel::Console => self.render_console_panel(ui),
            DevToolsPanel::Network => self.render_network_panel(ui),
            DevToolsPanel::Performance => self.render_performance_panel(ui),
            _ => {}
        }
    }

    fn render_console_panel(&self, ui: &mut egui::Ui) {
        ui.label("Console");
        // Console output, input field
    }
}
```

**Panels to Implement**:
1. **Elements**: DOM tree viewer (basic)
2. **Console**: JavaScript console output
3. **Network**: Request monitoring
4. **Performance**: Basic timing info
5. **Application**: Storage viewer

**Note**: Full DevTools require deep WebView integration. Initial implementation provides UI hosting and basic functionality.

**Testing**:
- Open/close DevTools
- Switch panels
- Dock/undock
- Tests: 12+

**Deliverables**:
- `components/devtools_host/` - new component
- Basic panel implementations
- UI integration in `ui_chrome`
- Tests: 12+

---

### Phase 8 Summary

**Version**: v0.7.0 → v0.8.0

**Deliverables**:
- ✅ Extension UI hosting
- ✅ DevTools hosting (basic)

**Tests Added**: ~27 tests
**Components Created**: `extension_host`, `devtools_host`
**Components Modified**: `ui_chrome`

---

## Phase 9: Production Hardening (v0.9.0 → v1.0.0)

**Goal**: Performance, security, reliability
**Duration**: Week 5-6
**Dependencies**: All previous phases complete

### 9.1: Performance Benchmarking

**Component**: `benches/` (new benchmark suite)

**Implementation**:
```rust
// benches/shell_performance.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_window_creation(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("create_window", |b| {
        b.iter(|| {
            runtime.block_on(async {
                let shell = create_test_shell().await;
                black_box(shell.create_window(WindowConfig::default()).await.unwrap());
            });
        });
    });
}

fn bench_tab_creation(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let shell = runtime.block_on(create_test_shell());
    let window_id = runtime.block_on(shell.create_window(WindowConfig::default())).unwrap();

    c.bench_function("create_tab", |b| {
        b.iter(|| {
            runtime.block_on(async {
                black_box(shell.create_tab(window_id, None).await.unwrap());
            });
        });
    });
}

fn bench_tab_switching(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let shell = runtime.block_on(create_test_shell());
    let window_id = runtime.block_on(shell.create_window(WindowConfig::default())).unwrap();

    // Create 10 tabs
    let tabs: Vec<_> = (0..10)
        .map(|_| runtime.block_on(shell.create_tab(window_id, None)).unwrap())
        .collect();

    let mut i = 0;
    c.bench_function("switch_tab", |b| {
        b.iter(|| {
            runtime.block_on(async {
                black_box(shell.activate_tab(tabs[i % 10]).await.unwrap());
            });
            i += 1;
        });
    });
}

fn bench_message_routing(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let shell = runtime.block_on(create_test_shell());

    c.bench_function("route_message", |b| {
        b.iter(|| {
            runtime.block_on(async {
                black_box(shell.handle_message(ComponentMessage::Heartbeat).await);
            });
        });
    });
}

criterion_group!(
    benches,
    bench_window_creation,
    bench_tab_creation,
    bench_tab_switching,
    bench_message_routing
);
criterion_main!(benches);
```

**Benchmarks**:
- Window creation (target: < 100ms)
- Tab creation (target: < 50ms)
- Tab switching (target: < 10ms)
- Message routing (target: < 1ms)
- UI frame render (target: < 16ms)

**Validation**:
- All benchmarks meet spec targets
- No performance regressions

**Deliverables**:
- Comprehensive benchmark suite
- Performance report
- Optimization where needed

---

### 9.2: Security Hardening

**Component**: `security/` (new module)

**Implementation**:
```rust
// components/security/src/ipc_validator.rs
pub struct IpcValidator {
    max_message_size: usize,
    allowed_sources: HashSet<ComponentId>,
}

impl IpcValidator {
    pub fn validate(&self, msg: &ComponentMessage) -> Result<(), SecurityError> {
        // Check message size
        if msg.size() > self.max_message_size {
            return Err(SecurityError::MessageTooLarge);
        }

        // Validate source
        if !self.allowed_sources.contains(&msg.source) {
            return Err(SecurityError::UnauthorizedSource);
        }

        // Validate payload
        self.validate_payload(&msg.payload)?;

        Ok(())
    }

    fn validate_payload(&self, payload: &MessagePayload) -> Result<(), SecurityError> {
        match payload {
            MessagePayload::Navigate { url, .. } => {
                // Validate URL
                self.validate_url(url)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn validate_url(&self, url: &str) -> Result<(), SecurityError> {
        // Block dangerous schemes
        if url.starts_with("javascript:") || url.starts_with("data:") {
            return Err(SecurityError::DangerousScheme);
        }

        // Validate URL format
        Url::parse(url).map_err(|_| SecurityError::InvalidUrl)?;

        Ok(())
    }
}

// components/security/src/input_sanitizer.rs
pub struct InputSanitizer;

impl InputSanitizer {
    pub fn sanitize_user_input(&self, input: &str) -> String {
        input.chars()
            .filter(|c| !c.is_control())
            .take(10000) // Max length
            .collect()
    }
}

// components/security/src/process_manager.rs
pub struct ProcessManager {
    tab_processes: HashMap<TabId, ProcessHandle>,
    max_processes: usize,
}

impl ProcessManager {
    pub fn spawn_tab_process(&mut self, tab_id: TabId) -> Result<ProcessHandle> {
        if self.tab_processes.len() >= self.max_processes {
            return Err(SecurityError::TooManyProcesses);
        }

        // Spawn isolated process for tab (platform-specific)
        let handle = self.spawn_isolated_process()?;
        self.tab_processes.insert(tab_id, handle);

        Ok(handle)
    }
}
```

**Security Features**:
- IPC message validation
- Input sanitization
- Process isolation (basic)
- URL validation
- Permission checks

**Testing**:
- Reject oversized messages
- Block dangerous URLs
- Validate all inputs
- Tests: 20+

**Deliverables**:
- `components/security/` - new component
- Integration with message bus
- Security tests: 20+

---

### 9.3: Crash Recovery

**Component**: `session_manager` (new)

**Implementation**:
```rust
// components/session_manager/src/lib.rs
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct BrowserSession {
    pub windows: Vec<WindowSession>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
pub struct WindowSession {
    pub id: WindowId,
    pub tabs: Vec<TabSession>,
    pub active_tab: Option<TabId>,
    pub position: (i32, i32),
    pub size: (u32, u32),
}

#[derive(Serialize, Deserialize)]
pub struct TabSession {
    pub id: TabId,
    pub url: String,
    pub title: String,
    pub history: Vec<String>,
}

pub struct SessionManager {
    current_session: BrowserSession,
    session_file: PathBuf,
}

impl SessionManager {
    pub fn save_session(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.current_session)?;
        std::fs::write(&self.session_file, json)?;
        Ok(())
    }

    pub fn load_session(&mut self) -> Result<BrowserSession> {
        let json = std::fs::read_to_string(&self.session_file)?;
        let session = serde_json::from_str(&json)?;
        Ok(session)
    }

    pub fn restore_session(&self) -> Result<()> {
        let session = self.load_session()?;

        for window in session.windows {
            // Recreate window
            // Recreate tabs
            // Restore URLs
        }

        Ok(())
    }

    pub fn auto_save(&self) {
        // Save session every 30 seconds
        // Save on tab/window changes
    }
}
```

**Features**:
- Auto-save session every 30 seconds
- Save on crashes
- Restore on startup
- "Restore previous session" dialog

**Testing**:
- Save session
- Restore session
- Handle corrupted session
- Tests: 12+

**Deliverables**:
- `components/session_manager/` - new component
- Crash recovery UI
- Tests: 12+

---

### 9.4: Auto-Update System

**Component**: `updater` (new)

**Implementation**:
```rust
// components/updater/src/lib.rs
use semver::Version;

pub struct Updater {
    current_version: Version,
    update_channel: UpdateChannel,
    update_url: String,
}

pub enum UpdateChannel {
    Stable,
    Beta,
    Dev,
}

pub struct UpdateInfo {
    pub version: Version,
    pub download_url: String,
    pub release_notes: String,
    pub checksum: String,
}

impl Updater {
    pub async fn check_for_updates(&self) -> Result<Option<UpdateInfo>> {
        // Fetch update manifest from server
        let manifest = self.fetch_update_manifest().await?;

        if manifest.version > self.current_version {
            Ok(Some(manifest))
        } else {
            Ok(None)
        }
    }

    pub async fn download_update(&self, info: &UpdateInfo) -> Result<PathBuf> {
        // Download update package
        let path = self.download_file(&info.download_url).await?;

        // Verify checksum
        self.verify_checksum(&path, &info.checksum)?;

        Ok(path)
    }

    pub fn install_update(&self, package: &Path) -> Result<()> {
        // Extract package
        // Replace executable
        // Restart browser

        #[cfg(target_os = "linux")]
        self.linux_install(package)?;

        #[cfg(target_os = "windows")]
        self.windows_install(package)?;

        #[cfg(target_os = "macos")]
        self.macos_install(package)?;

        Ok(())
    }
}
```

**Features**:
- Check for updates on startup
- Download updates in background
- Install on restart
- Update notifications
- Manual update check

**Testing**:
- Check for updates
- Download update
- Verify checksum
- Tests: 10+

**Deliverables**:
- `components/updater/` - new component
- Update UI in settings
- Tests: 10+

---

### Phase 9 Summary

**Version**: v0.8.0 → v0.9.0 → v1.0.0

**Deliverables**:
- ✅ Performance benchmarking
- ✅ Security hardening
- ✅ Crash recovery
- ✅ Auto-update system

**Tests Added**: ~54 tests
**Components Created**: `security`, `session_manager`, `updater`

**Milestone**: Production-ready browser shell!

---

## Overall Summary

### Timeline

| Phase | Duration | Version | Focus |
|-------|----------|---------|-------|
| Phase 5 | Week 1-2 | v0.5.0 | WebView, menus, tab DnD |
| Phase 6 | Week 2-3 | v0.6.0 | History, find, print |
| Phase 7 | Week 3-4 | v0.7.0 | Platform integration |
| Phase 8 | Week 4-5 | v0.8.0 | Extensions, DevTools |
| Phase 9 | Week 5-6 | v1.0.0 | Production hardening |

**Total Duration**: 5-6 weeks

### Test Coverage

| Phase | Tests Added | Cumulative |
|-------|-------------|------------|
| Current (v0.4.0) | - | 520 |
| Phase 5 | ~65 | 585 |
| Phase 6 | ~47 | 632 |
| Phase 7 | ~48 | 680 |
| Phase 8 | ~27 | 707 |
| Phase 9 | ~54 | 761 |

**Final Test Count**: ~760+ tests

### Components

**New Components**:
- `content_area` - WebView integration
- `history_manager` - History tracking
- `extension_host` - Extension UI
- `devtools_host` - DevTools UI
- `security` - Security validation
- `session_manager` - Crash recovery
- `updater` - Auto-updates

**Enhanced Components**:
- `ui_chrome` - Menus, find, print, DnD
- `platform_abstraction` - Notifications, clipboard, protocols
- `tab_manager` - Process isolation

### Completion Status

| Milestone | Before | After |
|-----------|--------|-------|
| Milestone 1: Basic Shell | 100% | 100% |
| Milestone 2: Browser Chrome | 80% | 100% |
| Milestone 3: Component Integration | 0% | 100% |
| Milestone 4: Advanced Features | 40% | 100% |
| Milestone 5: Platform Features | 0% | 100% |
| Milestone 6: Production Ready | 20% | 100% |

**Overall**: 50% → 100%

---

## Risk Assessment

### High Risk Items

1. **WebView Integration** (Phase 5)
   - Complexity: High
   - Risk: Integration challenges with egui
   - Mitigation: Use wry library (battle-tested), follow examples

2. **Platform-Specific Code** (Phase 7, 9)
   - Complexity: Medium
   - Risk: Platform differences
   - Mitigation: Use cross-platform libraries (notify-rust, arboard, etc.)

3. **Process Isolation** (Phase 9)
   - Complexity: Very High
   - Risk: Platform-specific, complex
   - Mitigation: Start with basic implementation, defer full sandboxing

### Medium Risk Items

1. **Performance Targets**
   - Risk: May not meet all spec targets initially
   - Mitigation: Benchmark early, optimize iteratively

2. **DevTools Integration**
   - Risk: Deep WebView integration needed
   - Mitigation: Start with basic UI, defer advanced features

---

## Success Criteria

### Phase 5 (v0.5.0)
- ✅ WebView displays web pages
- ✅ Full menu system functional
- ✅ Tab drag-and-drop works
- ✅ All tests passing

### Phase 6 (v0.6.0)
- ✅ History tracking works
- ✅ Find in page functional
- ✅ Print dialog and preview work
- ✅ All tests passing

### Phase 7 (v0.7.0)
- ✅ Notifications appear on all platforms
- ✅ Clipboard integration works
- ✅ Drag-and-drop files works
- ✅ File associations registered
- ✅ All tests passing

### Phase 8 (v0.8.0)
- ✅ Extension UI hosting works
- ✅ DevTools panels display
- ✅ All tests passing

### Phase 9 (v1.0.0)
- ✅ All benchmarks meet targets
- ✅ Security validation enabled
- ✅ Session restore works
- ✅ Auto-update functional
- ✅ All tests passing
- ✅ Zero compiler warnings
- ✅ Production-ready

---

## Next Steps

1. **Review this plan** with stakeholders
2. **Begin Phase 5** implementation
3. **Track progress** using TodoWrite
4. **Commit after each phase**
5. **Version bump** after each phase
6. **Generate completion reports** after each phase

---

**Plan Version**: 1.0
**Created**: 2025-11-14
**Status**: Ready to begin implementation
