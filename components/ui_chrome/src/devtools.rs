//! Developer Tools Panel for the browser
//!
//! Provides a Chrome-style developer tools interface with tabbed panels for:
//! - Elements: DOM tree inspection
//! - Console: JavaScript console log display
//! - Network: HTTP request/response inspection
//! - Sources: Source file viewer
//! - Performance: Performance profiling
//! - Memory: Memory usage analysis
//!
//! # Example
//!
//! ```rust,ignore
//! use ui_chrome::devtools::{DevToolsPanel, DevToolsConfig, DockPosition};
//!
//! let mut devtools = DevToolsPanel::new(DevToolsConfig::default());
//!
//! // Toggle visibility
//! devtools.toggle();
//!
//! // Change dock position
//! devtools.set_dock_position(DockPosition::Right);
//!
//! // Render in egui
//! devtools.show(&ctx);
//! ```

use egui::Color32;
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

/// Tab selection for the developer tools panel
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DevToolsTab {
    /// DOM tree inspector
    Elements,
    /// JavaScript console
    #[default]
    Console,
    /// Network request inspector
    Network,
    /// Source file viewer
    Sources,
    /// Performance profiler
    Performance,
    /// Memory analyzer
    Memory,
}

impl DevToolsTab {
    /// Get the display label for this tab
    pub fn label(&self) -> &'static str {
        match self {
            DevToolsTab::Elements => "Elements",
            DevToolsTab::Console => "Console",
            DevToolsTab::Network => "Network",
            DevToolsTab::Sources => "Sources",
            DevToolsTab::Performance => "Performance",
            DevToolsTab::Memory => "Memory",
        }
    }

    /// Get all available tabs in order
    pub fn all() -> &'static [DevToolsTab] {
        &[
            DevToolsTab::Elements,
            DevToolsTab::Console,
            DevToolsTab::Network,
            DevToolsTab::Sources,
            DevToolsTab::Performance,
            DevToolsTab::Memory,
        ]
    }
}

/// Dock position for the developer tools panel
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DockPosition {
    /// Docked at the bottom of the window
    #[default]
    Bottom,
    /// Docked on the right side of the window
    Right,
    /// Floating/detached window
    Detached,
}

impl DockPosition {
    /// Get the display label for this position
    pub fn label(&self) -> &'static str {
        match self {
            DockPosition::Bottom => "Bottom",
            DockPosition::Right => "Right",
            DockPosition::Detached => "Detached",
        }
    }
}

/// Console message severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ConsoleLevel {
    /// Regular log message
    #[default]
    Log,
    /// Informational message
    Info,
    /// Warning message
    Warn,
    /// Error message
    Error,
    /// Debug message
    Debug,
}

impl ConsoleLevel {
    /// Get the color for this log level
    pub fn color(&self) -> Color32 {
        match self {
            ConsoleLevel::Log => Color32::from_rgb(200, 200, 200),
            ConsoleLevel::Info => Color32::from_rgb(100, 180, 255),
            ConsoleLevel::Warn => Color32::from_rgb(255, 200, 100),
            ConsoleLevel::Error => Color32::from_rgb(255, 100, 100),
            ConsoleLevel::Debug => Color32::from_rgb(180, 180, 180),
        }
    }

    /// Get the prefix string for this log level
    pub fn prefix(&self) -> &'static str {
        match self {
            ConsoleLevel::Log => "",
            ConsoleLevel::Info => "[INFO]",
            ConsoleLevel::Warn => "[WARN]",
            ConsoleLevel::Error => "[ERROR]",
            ConsoleLevel::Debug => "[DEBUG]",
        }
    }
}

/// A single console message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleMessage {
    /// Message severity level
    pub level: ConsoleLevel,
    /// Message content
    pub message: String,
    /// Source file (if known)
    pub source: Option<String>,
    /// Line number in source (if known)
    pub line: Option<u32>,
    /// Timestamp when message was logged
    pub timestamp: u64,
}

impl ConsoleMessage {
    /// Create a new console message
    pub fn new(level: ConsoleLevel, message: impl Into<String>) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_millis() as u64;

        Self {
            level,
            message: message.into(),
            source: None,
            line: None,
            timestamp,
        }
    }

    /// Create a log message
    pub fn log(message: impl Into<String>) -> Self {
        Self::new(ConsoleLevel::Log, message)
    }

    /// Create an info message
    pub fn info(message: impl Into<String>) -> Self {
        Self::new(ConsoleLevel::Info, message)
    }

    /// Create a warning message
    pub fn warn(message: impl Into<String>) -> Self {
        Self::new(ConsoleLevel::Warn, message)
    }

    /// Create an error message
    pub fn error(message: impl Into<String>) -> Self {
        Self::new(ConsoleLevel::Error, message)
    }

    /// Create a debug message
    pub fn debug(message: impl Into<String>) -> Self {
        Self::new(ConsoleLevel::Debug, message)
    }

    /// Set the source file for this message
    pub fn with_source(mut self, source: impl Into<String>, line: u32) -> Self {
        self.source = Some(source.into());
        self.line = Some(line);
        self
    }
}

/// HTTP method for network requests
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum HttpMethod {
    #[default]
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    HEAD,
    OPTIONS,
}

impl HttpMethod {
    /// Get the display string for this method
    pub fn as_str(&self) -> &'static str {
        match self {
            HttpMethod::GET => "GET",
            HttpMethod::POST => "POST",
            HttpMethod::PUT => "PUT",
            HttpMethod::DELETE => "DELETE",
            HttpMethod::PATCH => "PATCH",
            HttpMethod::HEAD => "HEAD",
            HttpMethod::OPTIONS => "OPTIONS",
        }
    }

    /// Get the color for this method
    pub fn color(&self) -> Color32 {
        match self {
            HttpMethod::GET => Color32::from_rgb(100, 200, 100),
            HttpMethod::POST => Color32::from_rgb(255, 200, 100),
            HttpMethod::PUT => Color32::from_rgb(100, 180, 255),
            HttpMethod::DELETE => Color32::from_rgb(255, 100, 100),
            HttpMethod::PATCH => Color32::from_rgb(200, 150, 255),
            HttpMethod::HEAD => Color32::from_rgb(180, 180, 180),
            HttpMethod::OPTIONS => Color32::from_rgb(180, 180, 180),
        }
    }
}

/// Network request status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum NetworkStatus {
    /// Request is pending
    #[default]
    Pending,
    /// Request completed successfully with status code
    Complete(u16),
    /// Request failed with error message
    Failed(String),
    /// Request was cancelled
    Cancelled,
}

impl NetworkStatus {
    /// Check if the request is still pending
    pub fn is_pending(&self) -> bool {
        matches!(self, NetworkStatus::Pending)
    }

    /// Check if the request completed successfully (2xx status)
    pub fn is_success(&self) -> bool {
        matches!(self, NetworkStatus::Complete(code) if (200..300).contains(code))
    }

    /// Get the status color
    pub fn color(&self) -> Color32 {
        match self {
            NetworkStatus::Pending => Color32::from_rgb(255, 200, 100),
            NetworkStatus::Complete(code) if (200..300).contains(code) => {
                Color32::from_rgb(100, 200, 100)
            }
            NetworkStatus::Complete(code) if (300..400).contains(code) => {
                Color32::from_rgb(100, 180, 255)
            }
            NetworkStatus::Complete(_) => Color32::from_rgb(255, 100, 100),
            NetworkStatus::Failed(_) => Color32::from_rgb(255, 100, 100),
            NetworkStatus::Cancelled => Color32::from_rgb(180, 180, 180),
        }
    }
}

/// Timing information for a network request
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetworkTiming {
    /// Time to establish connection (ms)
    pub connect_time_ms: u64,
    /// Time to first byte (ms)
    pub ttfb_ms: u64,
    /// Total duration (ms)
    pub total_time_ms: u64,
    /// Download size in bytes
    pub download_size: u64,
}

/// A network request entry for the inspector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInspectorEntry {
    /// Unique request ID
    pub id: u64,
    /// HTTP method
    pub method: HttpMethod,
    /// Request URL
    pub url: String,
    /// Request status
    pub status: NetworkStatus,
    /// Response content type
    pub content_type: Option<String>,
    /// Timing information
    pub timing: NetworkTiming,
    /// Request headers
    pub request_headers: Vec<(String, String)>,
    /// Response headers
    pub response_headers: Vec<(String, String)>,
    /// Request body (if captured)
    pub request_body: Option<String>,
    /// Response body preview (truncated if large)
    pub response_preview: Option<String>,
    /// Timestamp when request started
    pub start_time: u64,
}

impl NetworkInspectorEntry {
    /// Create a new network entry for a request
    pub fn new(id: u64, method: HttpMethod, url: impl Into<String>) -> Self {
        let start_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_millis() as u64;

        Self {
            id,
            method,
            url: url.into(),
            status: NetworkStatus::Pending,
            content_type: None,
            timing: NetworkTiming::default(),
            request_headers: Vec::new(),
            response_headers: Vec::new(),
            request_body: None,
            response_preview: None,
            start_time,
        }
    }

    /// Mark the request as complete
    pub fn complete(&mut self, status_code: u16, content_type: Option<String>) {
        self.status = NetworkStatus::Complete(status_code);
        self.content_type = content_type;
    }

    /// Mark the request as failed
    pub fn fail(&mut self, error: impl Into<String>) {
        self.status = NetworkStatus::Failed(error.into());
    }

    /// Get a short display name for the URL (last path segment or domain)
    pub fn display_name(&self) -> String {
        if let Some(path) = self.url.split('/').last() {
            if !path.is_empty() && !path.contains('?') {
                return path.to_string();
            }
        }
        // Fall back to domain
        self.url
            .split("://")
            .nth(1)
            .and_then(|s| s.split('/').next())
            .unwrap_or(&self.url)
            .to_string()
    }
}

/// Configuration for the developer tools panel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevToolsConfig {
    /// Default panel height/width (depending on dock position)
    pub default_size: f32,
    /// Minimum panel size
    pub min_size: f32,
    /// Maximum panel size
    pub max_size: f32,
    /// Tab order (customizable)
    pub tab_order: Vec<DevToolsTab>,
    /// Whether to preserve logs on navigation
    pub preserve_log: bool,
    /// Maximum number of console messages to keep
    pub max_console_messages: usize,
    /// Maximum number of network entries to keep
    pub max_network_entries: usize,
    /// Console filter text
    pub console_filter: String,
    /// Network filter text
    pub network_filter: String,
}

impl Default for DevToolsConfig {
    fn default() -> Self {
        Self {
            default_size: 300.0,
            min_size: 150.0,
            max_size: 600.0,
            tab_order: DevToolsTab::all().to_vec(),
            preserve_log: false,
            max_console_messages: 1000,
            max_network_entries: 500,
            console_filter: String::new(),
            network_filter: String::new(),
        }
    }
}

/// Current state of the developer tools panel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevToolsState {
    /// Currently selected tab
    pub current_tab: DevToolsTab,
    /// Current dock position
    pub dock_position: DockPosition,
    /// Current panel size
    pub panel_size: f32,
    /// Whether the panel is visible
    pub visible: bool,
    /// Selected network entry ID (for detail view)
    pub selected_network_entry: Option<u64>,
    /// Whether console is scrolled to bottom
    pub console_auto_scroll: bool,
}

impl Default for DevToolsState {
    fn default() -> Self {
        Self {
            current_tab: DevToolsTab::Console,
            dock_position: DockPosition::Bottom,
            panel_size: 300.0,
            visible: false,
            selected_network_entry: None,
            console_auto_scroll: true,
        }
    }
}

/// Developer tools panel component
#[derive(Debug)]
pub struct DevToolsPanel {
    /// Configuration settings
    config: DevToolsConfig,
    /// Current state
    state: DevToolsState,
    /// Console messages
    console_messages: Vec<ConsoleMessage>,
    /// Network entries
    network_entries: Vec<NetworkInspectorEntry>,
    /// Next network entry ID
    next_network_id: u64,
}

impl DevToolsPanel {
    /// Create a new developer tools panel with the given configuration
    pub fn new(config: DevToolsConfig) -> Self {
        let panel_size = config.default_size;
        Self {
            config,
            state: DevToolsState {
                panel_size,
                ..Default::default()
            },
            console_messages: Vec::new(),
            network_entries: Vec::new(),
            next_network_id: 1,
        }
    }

    /// Get the current state
    pub fn state(&self) -> &DevToolsState {
        &self.state
    }

    /// Get the configuration
    pub fn config(&self) -> &DevToolsConfig {
        &self.config
    }

    /// Check if the panel is visible
    pub fn is_visible(&self) -> bool {
        self.state.visible
    }

    /// Show the panel
    pub fn show_panel(&mut self) {
        self.state.visible = true;
    }

    /// Hide the panel
    pub fn hide(&mut self) {
        self.state.visible = false;
    }

    /// Toggle panel visibility
    pub fn toggle(&mut self) {
        self.state.visible = !self.state.visible;
    }

    /// Get the current tab
    pub fn current_tab(&self) -> DevToolsTab {
        self.state.current_tab
    }

    /// Set the current tab
    pub fn set_tab(&mut self, tab: DevToolsTab) {
        self.state.current_tab = tab;
    }

    /// Get the dock position
    pub fn dock_position(&self) -> DockPosition {
        self.state.dock_position
    }

    /// Set the dock position
    pub fn set_dock_position(&mut self, position: DockPosition) {
        self.state.dock_position = position;
    }

    /// Get the panel size
    pub fn panel_size(&self) -> f32 {
        self.state.panel_size
    }

    /// Set the panel size (clamped to config bounds)
    pub fn set_panel_size(&mut self, size: f32) {
        self.state.panel_size = size.clamp(self.config.min_size, self.config.max_size);
    }

    // Console methods

    /// Add a console message
    pub fn add_console_message(&mut self, message: ConsoleMessage) {
        self.console_messages.push(message);

        // Trim old messages if over limit
        if self.console_messages.len() > self.config.max_console_messages {
            let excess = self.console_messages.len() - self.config.max_console_messages;
            self.console_messages.drain(0..excess);
        }
    }

    /// Log a message to the console
    pub fn console_log(&mut self, message: impl Into<String>) {
        self.add_console_message(ConsoleMessage::log(message));
    }

    /// Log an info message to the console
    pub fn console_info(&mut self, message: impl Into<String>) {
        self.add_console_message(ConsoleMessage::info(message));
    }

    /// Log a warning to the console
    pub fn console_warn(&mut self, message: impl Into<String>) {
        self.add_console_message(ConsoleMessage::warn(message));
    }

    /// Log an error to the console
    pub fn console_error(&mut self, message: impl Into<String>) {
        self.add_console_message(ConsoleMessage::error(message));
    }

    /// Log a debug message to the console
    pub fn console_debug(&mut self, message: impl Into<String>) {
        self.add_console_message(ConsoleMessage::debug(message));
    }

    /// Clear all console messages
    pub fn clear_console(&mut self) {
        self.console_messages.clear();
    }

    /// Get all console messages
    pub fn console_messages(&self) -> &[ConsoleMessage] {
        &self.console_messages
    }

    /// Get console messages matching the current filter
    pub fn filtered_console_messages(&self) -> Vec<&ConsoleMessage> {
        if self.config.console_filter.is_empty() {
            self.console_messages.iter().collect()
        } else {
            let filter = self.config.console_filter.to_lowercase();
            self.console_messages
                .iter()
                .filter(|m| m.message.to_lowercase().contains(&filter))
                .collect()
        }
    }

    /// Set the console filter
    pub fn set_console_filter(&mut self, filter: impl Into<String>) {
        self.config.console_filter = filter.into();
    }

    // Network methods

    /// Start tracking a new network request
    pub fn add_network_request(&mut self, method: HttpMethod, url: impl Into<String>) -> u64 {
        let id = self.next_network_id;
        self.next_network_id += 1;

        let entry = NetworkInspectorEntry::new(id, method, url);
        self.network_entries.push(entry);

        // Trim old entries if over limit
        if self.network_entries.len() > self.config.max_network_entries {
            let excess = self.network_entries.len() - self.config.max_network_entries;
            self.network_entries.drain(0..excess);
        }

        id
    }

    /// Update a network request with completion info
    pub fn complete_network_request(
        &mut self,
        id: u64,
        status_code: u16,
        content_type: Option<String>,
    ) {
        if let Some(entry) = self.network_entries.iter_mut().find(|e| e.id == id) {
            entry.complete(status_code, content_type);
        }
    }

    /// Mark a network request as failed
    pub fn fail_network_request(&mut self, id: u64, error: impl Into<String>) {
        if let Some(entry) = self.network_entries.iter_mut().find(|e| e.id == id) {
            entry.fail(error);
        }
    }

    /// Clear all network entries
    pub fn clear_network(&mut self) {
        self.network_entries.clear();
    }

    /// Get all network entries
    pub fn network_entries(&self) -> &[NetworkInspectorEntry] {
        &self.network_entries
    }

    /// Get network entries matching the current filter
    pub fn filtered_network_entries(&self) -> Vec<&NetworkInspectorEntry> {
        if self.config.network_filter.is_empty() {
            self.network_entries.iter().collect()
        } else {
            let filter = self.config.network_filter.to_lowercase();
            self.network_entries
                .iter()
                .filter(|e| e.url.to_lowercase().contains(&filter))
                .collect()
        }
    }

    /// Set the network filter
    pub fn set_network_filter(&mut self, filter: impl Into<String>) {
        self.config.network_filter = filter.into();
    }

    /// Select a network entry for detail view
    pub fn select_network_entry(&mut self, id: Option<u64>) {
        self.state.selected_network_entry = id;
    }

    /// Clear on navigation (if preserve_log is false)
    pub fn on_navigation(&mut self) {
        if !self.config.preserve_log {
            self.clear_console();
            self.clear_network();
        }
    }

    /// Show the developer tools panel in egui
    pub fn show(&mut self, ctx: &egui::Context) {
        if !self.state.visible {
            return;
        }

        match self.state.dock_position {
            DockPosition::Bottom => self.show_bottom_panel(ctx),
            DockPosition::Right => self.show_right_panel(ctx),
            DockPosition::Detached => self.show_detached_window(ctx),
        }
    }

    /// Render as a bottom panel
    fn show_bottom_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("devtools_panel")
            .resizable(true)
            .min_height(self.config.min_size)
            .max_height(self.config.max_size)
            .default_height(self.state.panel_size)
            .show(ctx, |ui| {
                self.render_panel_content(ui);
            });
    }

    /// Render as a right panel
    fn show_right_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("devtools_panel")
            .resizable(true)
            .min_width(self.config.min_size)
            .max_width(self.config.max_size)
            .default_width(self.state.panel_size)
            .show(ctx, |ui| {
                self.render_panel_content(ui);
            });
    }

    /// Render as a detached window
    fn show_detached_window(&mut self, ctx: &egui::Context) {
        egui::Window::new("Developer Tools")
            .default_size([600.0, 400.0])
            .resizable(true)
            .collapsible(true)
            .show(ctx, |ui| {
                self.render_panel_content(ui);
            });
    }

    /// Render the panel content (tabs + content area)
    fn render_panel_content(&mut self, ui: &mut egui::Ui) {
        // Tab bar
        ui.horizontal(|ui| {
            for tab in &self.config.tab_order {
                let selected = self.state.current_tab == *tab;
                if ui.selectable_label(selected, tab.label()).clicked() {
                    self.state.current_tab = *tab;
                }
            }

            // Right-aligned controls
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Close button
                if ui.small_button("✕").clicked() {
                    self.state.visible = false;
                }

                // Dock position selector
                egui::ComboBox::from_id_salt("dock_position")
                    .selected_text(self.state.dock_position.label())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.state.dock_position,
                            DockPosition::Bottom,
                            "Bottom",
                        );
                        ui.selectable_value(
                            &mut self.state.dock_position,
                            DockPosition::Right,
                            "Right",
                        );
                        ui.selectable_value(
                            &mut self.state.dock_position,
                            DockPosition::Detached,
                            "Detached",
                        );
                    });
            });
        });

        ui.separator();

        // Content area based on selected tab
        match self.state.current_tab {
            DevToolsTab::Elements => self.render_elements_panel(ui),
            DevToolsTab::Console => self.render_console_panel(ui),
            DevToolsTab::Network => self.render_network_panel(ui),
            DevToolsTab::Sources => self.render_sources_panel(ui),
            DevToolsTab::Performance => self.render_performance_panel(ui),
            DevToolsTab::Memory => self.render_memory_panel(ui),
        }
    }

    /// Render the Elements panel (placeholder)
    fn render_elements_panel(&self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.heading("Elements Inspector");
            ui.label("DOM tree inspection will be available here.");
            ui.add_space(10.0);
            ui.label("Select an element to inspect its properties.");
        });
    }

    /// Render the Console panel
    fn render_console_panel(&mut self, ui: &mut egui::Ui) {
        // Console toolbar
        ui.horizontal(|ui| {
            if ui.button("Clear").clicked() {
                self.clear_console();
            }

            ui.separator();

            // Filter input
            ui.label("Filter:");
            let mut filter = self.config.console_filter.clone();
            if ui.text_edit_singleline(&mut filter).changed() {
                self.config.console_filter = filter;
            }

            ui.separator();

            ui.checkbox(&mut self.state.console_auto_scroll, "Auto-scroll");
        });

        ui.separator();

        // Console messages
        let messages = self.filtered_console_messages();
        let scroll_area = egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .stick_to_bottom(self.state.console_auto_scroll);

        scroll_area.show(ui, |ui| {
            for message in messages {
                ui.horizontal(|ui| {
                    // Level prefix with color
                    if !message.level.prefix().is_empty() {
                        ui.colored_label(message.level.color(), message.level.prefix());
                    }

                    // Message content
                    ui.colored_label(message.level.color(), &message.message);

                    // Source location (if available)
                    if let Some(source) = &message.source {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let location = if let Some(line) = message.line {
                                format!("{}:{}", source, line)
                            } else {
                                source.clone()
                            };
                            ui.small(location);
                        });
                    }
                });
            }

            if self.console_messages.is_empty() {
                ui.label("No console messages.");
            }
        });
    }

    /// Render the Network panel
    fn render_network_panel(&mut self, ui: &mut egui::Ui) {
        // Network toolbar
        ui.horizontal(|ui| {
            if ui.button("Clear").clicked() {
                self.clear_network();
            }

            ui.separator();

            // Filter input
            ui.label("Filter:");
            let mut filter = self.config.network_filter.clone();
            if ui.text_edit_singleline(&mut filter).changed() {
                self.config.network_filter = filter;
            }

            ui.separator();

            ui.checkbox(&mut self.config.preserve_log, "Preserve log");
        });

        ui.separator();

        // Collect entry display data to avoid borrow issues
        let filter = self.config.network_filter.to_lowercase();
        let entries_data: Vec<_> = self
            .network_entries
            .iter()
            .filter(|e| filter.is_empty() || e.url.to_lowercase().contains(&filter))
            .map(|e| {
                (
                    e.id,
                    e.method,
                    e.status.clone(),
                    e.display_name(),
                    e.timing.download_size,
                    e.url.clone(),
                    e.content_type.clone(),
                    e.timing.connect_time_ms,
                    e.timing.ttfb_ms,
                    e.timing.total_time_ms,
                )
            })
            .collect();

        let selected_id = self.state.selected_network_entry;
        let mut new_selection: Option<u64> = None;

        ui.columns(2, |columns| {
            // Request list
            egui::ScrollArea::vertical()
                .id_salt("network_list")
                .show(&mut columns[0], |ui| {
                    for (id, method, status, display_name, download_size, ..) in &entries_data {
                        let selected = selected_id == Some(*id);
                        ui.horizontal(|ui| {
                            // Status indicator
                            ui.colored_label(status.color(), "●");

                            // Method
                            ui.colored_label(method.color(), method.as_str());

                            // URL/name
                            let response = ui.selectable_label(selected, display_name);
                            if response.clicked() {
                                new_selection = Some(*id);
                            }

                            // Size (if available)
                            if *download_size > 0 {
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.small(format_size(*download_size));
                                    },
                                );
                            }
                        });
                    }

                    if entries_data.is_empty() {
                        ui.label("No network requests.");
                    }
                });

            // Details panel
            columns[1].vertical(|ui| {
                if let Some(id) = selected_id {
                    if let Some((
                        _,
                        method,
                        status,
                        _,
                        download_size,
                        url,
                        content_type,
                        connect_time,
                        ttfb,
                        total_time,
                    )) = entries_data.iter().find(|(eid, ..)| *eid == id)
                    {
                        ui.heading("Request Details");
                        ui.separator();

                        ui.horizontal(|ui| {
                            ui.strong("URL:");
                            ui.label(url);
                        });

                        ui.horizontal(|ui| {
                            ui.strong("Method:");
                            ui.colored_label(method.color(), method.as_str());
                        });

                        ui.horizontal(|ui| {
                            ui.strong("Status:");
                            let status_text = match status {
                                NetworkStatus::Pending => "Pending".to_string(),
                                NetworkStatus::Complete(code) => code.to_string(),
                                NetworkStatus::Failed(err) => format!("Failed: {}", err),
                                NetworkStatus::Cancelled => "Cancelled".to_string(),
                            };
                            ui.colored_label(status.color(), status_text);
                        });

                        if let Some(ct) = content_type {
                            ui.horizontal(|ui| {
                                ui.strong("Content-Type:");
                                ui.label(ct);
                            });
                        }

                        ui.separator();
                        ui.strong("Timing:");
                        ui.label(format!("Connect: {}ms", connect_time));
                        ui.label(format!("TTFB: {}ms", ttfb));
                        ui.label(format!("Total: {}ms", total_time));
                        ui.label(format!("Size: {}", format_size(*download_size)));
                    }
                } else {
                    ui.label("Select a request to view details.");
                }
            });
        });

        // Apply selection changes after the closure
        if let Some(id) = new_selection {
            self.state.selected_network_entry = Some(id);
        }
    }

    /// Render the Sources panel (placeholder)
    fn render_sources_panel(&self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.heading("Sources");
            ui.label("Source file viewer will be available here.");
            ui.add_space(10.0);
            ui.label("Browse and debug source files.");
        });
    }

    /// Render the Performance panel (placeholder)
    fn render_performance_panel(&self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.heading("Performance");
            ui.label("Performance profiling will be available here.");
            ui.add_space(10.0);
            ui.label("Record and analyze runtime performance.");
        });
    }

    /// Render the Memory panel (placeholder)
    fn render_memory_panel(&self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.heading("Memory");
            ui.label("Memory analyzer will be available here.");
            ui.add_space(10.0);
            ui.label("Track memory usage and detect leaks.");
        });
    }
}

impl Default for DevToolsPanel {
    fn default() -> Self {
        Self::new(DevToolsConfig::default())
    }
}

/// Format a byte size as human-readable string
fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // DevToolsTab tests
    #[test]
    fn test_devtools_tab_default() {
        let tab = DevToolsTab::default();
        assert_eq!(tab, DevToolsTab::Console);
    }

    #[test]
    fn test_devtools_tab_label() {
        assert_eq!(DevToolsTab::Elements.label(), "Elements");
        assert_eq!(DevToolsTab::Console.label(), "Console");
        assert_eq!(DevToolsTab::Network.label(), "Network");
        assert_eq!(DevToolsTab::Sources.label(), "Sources");
        assert_eq!(DevToolsTab::Performance.label(), "Performance");
        assert_eq!(DevToolsTab::Memory.label(), "Memory");
    }

    #[test]
    fn test_devtools_tab_all() {
        let all = DevToolsTab::all();
        assert_eq!(all.len(), 6);
        assert!(all.contains(&DevToolsTab::Elements));
        assert!(all.contains(&DevToolsTab::Console));
        assert!(all.contains(&DevToolsTab::Network));
    }

    // DockPosition tests
    #[test]
    fn test_dock_position_default() {
        let pos = DockPosition::default();
        assert_eq!(pos, DockPosition::Bottom);
    }

    #[test]
    fn test_dock_position_label() {
        assert_eq!(DockPosition::Bottom.label(), "Bottom");
        assert_eq!(DockPosition::Right.label(), "Right");
        assert_eq!(DockPosition::Detached.label(), "Detached");
    }

    // ConsoleLevel tests
    #[test]
    fn test_console_level_default() {
        let level = ConsoleLevel::default();
        assert_eq!(level, ConsoleLevel::Log);
    }

    #[test]
    fn test_console_level_prefix() {
        assert_eq!(ConsoleLevel::Log.prefix(), "");
        assert_eq!(ConsoleLevel::Info.prefix(), "[INFO]");
        assert_eq!(ConsoleLevel::Warn.prefix(), "[WARN]");
        assert_eq!(ConsoleLevel::Error.prefix(), "[ERROR]");
        assert_eq!(ConsoleLevel::Debug.prefix(), "[DEBUG]");
    }

    #[test]
    fn test_console_level_colors_are_different() {
        let colors: Vec<_> = [
            ConsoleLevel::Log,
            ConsoleLevel::Info,
            ConsoleLevel::Warn,
            ConsoleLevel::Error,
        ]
        .iter()
        .map(|l| l.color())
        .collect();

        // Warn, Error, Info should have distinct colors
        assert_ne!(colors[1], colors[2]); // Info != Warn
        assert_ne!(colors[2], colors[3]); // Warn != Error
    }

    // ConsoleMessage tests
    #[test]
    fn test_console_message_new() {
        let msg = ConsoleMessage::new(ConsoleLevel::Info, "test message");
        assert_eq!(msg.level, ConsoleLevel::Info);
        assert_eq!(msg.message, "test message");
        assert!(msg.source.is_none());
        assert!(msg.line.is_none());
        assert!(msg.timestamp > 0);
    }

    #[test]
    fn test_console_message_constructors() {
        let log = ConsoleMessage::log("log");
        assert_eq!(log.level, ConsoleLevel::Log);

        let info = ConsoleMessage::info("info");
        assert_eq!(info.level, ConsoleLevel::Info);

        let warn = ConsoleMessage::warn("warn");
        assert_eq!(warn.level, ConsoleLevel::Warn);

        let error = ConsoleMessage::error("error");
        assert_eq!(error.level, ConsoleLevel::Error);

        let debug = ConsoleMessage::debug("debug");
        assert_eq!(debug.level, ConsoleLevel::Debug);
    }

    #[test]
    fn test_console_message_with_source() {
        let msg = ConsoleMessage::log("test").with_source("main.js", 42);
        assert_eq!(msg.source, Some("main.js".to_string()));
        assert_eq!(msg.line, Some(42));
    }

    // HttpMethod tests
    #[test]
    fn test_http_method_default() {
        let method = HttpMethod::default();
        assert_eq!(method, HttpMethod::GET);
    }

    #[test]
    fn test_http_method_as_str() {
        assert_eq!(HttpMethod::GET.as_str(), "GET");
        assert_eq!(HttpMethod::POST.as_str(), "POST");
        assert_eq!(HttpMethod::PUT.as_str(), "PUT");
        assert_eq!(HttpMethod::DELETE.as_str(), "DELETE");
        assert_eq!(HttpMethod::PATCH.as_str(), "PATCH");
        assert_eq!(HttpMethod::HEAD.as_str(), "HEAD");
        assert_eq!(HttpMethod::OPTIONS.as_str(), "OPTIONS");
    }

    // NetworkStatus tests
    #[test]
    fn test_network_status_default() {
        let status = NetworkStatus::default();
        assert_eq!(status, NetworkStatus::Pending);
        assert!(status.is_pending());
    }

    #[test]
    fn test_network_status_is_success() {
        assert!(NetworkStatus::Complete(200).is_success());
        assert!(NetworkStatus::Complete(201).is_success());
        assert!(NetworkStatus::Complete(299).is_success());
        assert!(!NetworkStatus::Complete(300).is_success());
        assert!(!NetworkStatus::Complete(404).is_success());
        assert!(!NetworkStatus::Complete(500).is_success());
        assert!(!NetworkStatus::Pending.is_success());
        assert!(!NetworkStatus::Failed("error".to_string()).is_success());
    }

    // NetworkInspectorEntry tests
    #[test]
    fn test_network_entry_new() {
        let entry = NetworkInspectorEntry::new(1, HttpMethod::GET, "https://example.com/api");
        assert_eq!(entry.id, 1);
        assert_eq!(entry.method, HttpMethod::GET);
        assert_eq!(entry.url, "https://example.com/api");
        assert!(entry.status.is_pending());
        assert!(entry.start_time > 0);
    }

    #[test]
    fn test_network_entry_complete() {
        let mut entry = NetworkInspectorEntry::new(1, HttpMethod::GET, "https://example.com");
        entry.complete(200, Some("application/json".to_string()));

        assert_eq!(entry.status, NetworkStatus::Complete(200));
        assert_eq!(entry.content_type, Some("application/json".to_string()));
    }

    #[test]
    fn test_network_entry_fail() {
        let mut entry = NetworkInspectorEntry::new(1, HttpMethod::GET, "https://example.com");
        entry.fail("Connection refused");

        assert_eq!(
            entry.status,
            NetworkStatus::Failed("Connection refused".to_string())
        );
    }

    #[test]
    fn test_network_entry_display_name() {
        let entry1 = NetworkInspectorEntry::new(1, HttpMethod::GET, "https://example.com/api/users");
        assert_eq!(entry1.display_name(), "users");

        let entry2 = NetworkInspectorEntry::new(2, HttpMethod::GET, "https://example.com/");
        assert_eq!(entry2.display_name(), "example.com");

        let entry3 = NetworkInspectorEntry::new(3, HttpMethod::GET, "https://example.com");
        assert_eq!(entry3.display_name(), "example.com");
    }

    // DevToolsConfig tests
    #[test]
    fn test_devtools_config_default() {
        let config = DevToolsConfig::default();
        assert_eq!(config.default_size, 300.0);
        assert_eq!(config.min_size, 150.0);
        assert_eq!(config.max_size, 600.0);
        assert_eq!(config.tab_order.len(), 6);
        assert!(!config.preserve_log);
        assert_eq!(config.max_console_messages, 1000);
        assert_eq!(config.max_network_entries, 500);
    }

    // DevToolsState tests
    #[test]
    fn test_devtools_state_default() {
        let state = DevToolsState::default();
        assert_eq!(state.current_tab, DevToolsTab::Console);
        assert_eq!(state.dock_position, DockPosition::Bottom);
        assert_eq!(state.panel_size, 300.0);
        assert!(!state.visible);
        assert!(state.selected_network_entry.is_none());
        assert!(state.console_auto_scroll);
    }

    // DevToolsPanel tests
    #[test]
    fn test_devtools_panel_new() {
        let panel = DevToolsPanel::new(DevToolsConfig::default());
        assert!(!panel.is_visible());
        assert_eq!(panel.current_tab(), DevToolsTab::Console);
        assert_eq!(panel.dock_position(), DockPosition::Bottom);
    }

    #[test]
    fn test_devtools_panel_visibility() {
        let mut panel = DevToolsPanel::default();

        assert!(!panel.is_visible());

        panel.show_panel();
        assert!(panel.is_visible());

        panel.hide();
        assert!(!panel.is_visible());

        panel.toggle();
        assert!(panel.is_visible());

        panel.toggle();
        assert!(!panel.is_visible());
    }

    #[test]
    fn test_devtools_panel_tab_switching() {
        let mut panel = DevToolsPanel::default();

        panel.set_tab(DevToolsTab::Network);
        assert_eq!(panel.current_tab(), DevToolsTab::Network);

        panel.set_tab(DevToolsTab::Elements);
        assert_eq!(panel.current_tab(), DevToolsTab::Elements);
    }

    #[test]
    fn test_devtools_panel_dock_position() {
        let mut panel = DevToolsPanel::default();

        panel.set_dock_position(DockPosition::Right);
        assert_eq!(panel.dock_position(), DockPosition::Right);

        panel.set_dock_position(DockPosition::Detached);
        assert_eq!(panel.dock_position(), DockPosition::Detached);
    }

    #[test]
    fn test_devtools_panel_size_clamping() {
        let mut panel = DevToolsPanel::default();

        panel.set_panel_size(100.0); // Below min
        assert_eq!(panel.panel_size(), 150.0);

        panel.set_panel_size(1000.0); // Above max
        assert_eq!(panel.panel_size(), 600.0);

        panel.set_panel_size(400.0); // Within bounds
        assert_eq!(panel.panel_size(), 400.0);
    }

    #[test]
    fn test_devtools_console_messages() {
        let mut panel = DevToolsPanel::default();

        panel.console_log("log message");
        panel.console_info("info message");
        panel.console_warn("warn message");
        panel.console_error("error message");
        panel.console_debug("debug message");

        assert_eq!(panel.console_messages().len(), 5);
        assert_eq!(panel.console_messages()[0].level, ConsoleLevel::Log);
        assert_eq!(panel.console_messages()[1].level, ConsoleLevel::Info);
        assert_eq!(panel.console_messages()[2].level, ConsoleLevel::Warn);
        assert_eq!(panel.console_messages()[3].level, ConsoleLevel::Error);
        assert_eq!(panel.console_messages()[4].level, ConsoleLevel::Debug);
    }

    #[test]
    fn test_devtools_console_clear() {
        let mut panel = DevToolsPanel::default();

        panel.console_log("test");
        panel.console_log("test2");
        assert_eq!(panel.console_messages().len(), 2);

        panel.clear_console();
        assert_eq!(panel.console_messages().len(), 0);
    }

    #[test]
    fn test_devtools_console_filter() {
        let mut panel = DevToolsPanel::default();

        panel.console_log("hello world");
        panel.console_log("foo bar");
        panel.console_error("hello error");

        panel.set_console_filter("hello");
        let filtered = panel.filtered_console_messages();
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_devtools_console_message_limit() {
        let mut config = DevToolsConfig::default();
        config.max_console_messages = 5;
        let mut panel = DevToolsPanel::new(config);

        for i in 0..10 {
            panel.console_log(format!("message {}", i));
        }

        assert_eq!(panel.console_messages().len(), 5);
        // Should have messages 5-9 (oldest trimmed)
        assert!(panel.console_messages()[0].message.contains("5"));
    }

    #[test]
    fn test_devtools_network_request() {
        let mut panel = DevToolsPanel::default();

        let id = panel.add_network_request(HttpMethod::GET, "https://example.com/api");
        assert_eq!(id, 1);
        assert_eq!(panel.network_entries().len(), 1);

        let entry = &panel.network_entries()[0];
        assert_eq!(entry.method, HttpMethod::GET);
        assert!(entry.status.is_pending());
    }

    #[test]
    fn test_devtools_network_complete() {
        let mut panel = DevToolsPanel::default();

        let id = panel.add_network_request(HttpMethod::POST, "https://example.com/users");
        panel.complete_network_request(id, 201, Some("application/json".to_string()));

        let entry = &panel.network_entries()[0];
        assert_eq!(entry.status, NetworkStatus::Complete(201));
        assert_eq!(entry.content_type, Some("application/json".to_string()));
    }

    #[test]
    fn test_devtools_network_fail() {
        let mut panel = DevToolsPanel::default();

        let id = panel.add_network_request(HttpMethod::GET, "https://example.com");
        panel.fail_network_request(id, "Network error");

        let entry = &panel.network_entries()[0];
        assert_eq!(
            entry.status,
            NetworkStatus::Failed("Network error".to_string())
        );
    }

    #[test]
    fn test_devtools_network_clear() {
        let mut panel = DevToolsPanel::default();

        panel.add_network_request(HttpMethod::GET, "https://example.com");
        panel.add_network_request(HttpMethod::POST, "https://example.com/api");
        assert_eq!(panel.network_entries().len(), 2);

        panel.clear_network();
        assert_eq!(panel.network_entries().len(), 0);
    }

    #[test]
    fn test_devtools_network_filter() {
        let mut panel = DevToolsPanel::default();

        panel.add_network_request(HttpMethod::GET, "https://api.example.com/users");
        panel.add_network_request(HttpMethod::GET, "https://cdn.example.com/image.png");
        panel.add_network_request(HttpMethod::POST, "https://api.example.com/login");

        panel.set_network_filter("api");
        let filtered = panel.filtered_network_entries();
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_devtools_network_entry_limit() {
        let mut config = DevToolsConfig::default();
        config.max_network_entries = 3;
        let mut panel = DevToolsPanel::new(config);

        for i in 0..5 {
            panel.add_network_request(HttpMethod::GET, format!("https://example.com/{}", i));
        }

        assert_eq!(panel.network_entries().len(), 3);
        // Should have entries 2-4 (oldest trimmed)
        assert!(panel.network_entries()[0].url.contains("/2"));
    }

    #[test]
    fn test_devtools_on_navigation_clears() {
        let mut panel = DevToolsPanel::default();

        panel.console_log("test");
        panel.add_network_request(HttpMethod::GET, "https://example.com");

        panel.on_navigation();

        assert_eq!(panel.console_messages().len(), 0);
        assert_eq!(panel.network_entries().len(), 0);
    }

    #[test]
    fn test_devtools_on_navigation_preserves_when_enabled() {
        let mut config = DevToolsConfig::default();
        config.preserve_log = true;
        let mut panel = DevToolsPanel::new(config);

        panel.console_log("test");
        panel.add_network_request(HttpMethod::GET, "https://example.com");

        panel.on_navigation();

        assert_eq!(panel.console_messages().len(), 1);
        assert_eq!(panel.network_entries().len(), 1);
    }

    #[test]
    fn test_devtools_network_selection() {
        let mut panel = DevToolsPanel::default();

        let id1 = panel.add_network_request(HttpMethod::GET, "https://example.com/1");
        let _id2 = panel.add_network_request(HttpMethod::GET, "https://example.com/2");

        assert!(panel.state().selected_network_entry.is_none());

        panel.select_network_entry(Some(id1));
        assert_eq!(panel.state().selected_network_entry, Some(id1));

        panel.select_network_entry(None);
        assert!(panel.state().selected_network_entry.is_none());
    }

    // format_size tests
    #[test]
    fn test_format_size() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1536), "1.5 KB");
        assert_eq!(format_size(1048576), "1.0 MB");
        assert_eq!(format_size(1073741824), "1.0 GB");
    }
}
