//! Application menu system
//!
//! Provides the complete menu bar with File, Edit, View, History, Bookmarks, Tools, and Help menus.
//! Menu items emit messages to the message bus when clicked.

use message_bus::ComponentMessage;

/// Action to be taken when a menu item is clicked
#[derive(Debug, Clone)]
pub enum MenuAction {
    /// Emit a message to the message bus
    SendMessage(ComponentMessage),
    /// Toggle a UI panel
    TogglePanel(PanelType),
    /// Perform a UI action
    UiAction(UiAction),
    /// No action (disabled menu item)
    None,
}

/// Types of UI panels that can be toggled
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelType {
    Settings,
    History,
    Downloads,
    Bookmarks,
    DevTools,
}

/// UI actions that don't involve messaging
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiAction {
    ZoomIn,
    ZoomOut,
    ResetZoom,
    FullScreen,
    BookmarkPage,
    Find,
    About,
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    SelectAll,
    ShowAllHistory,
    ClearHistory,
    ShowAllBookmarks,
    ReportIssue,
    ShowDocumentation,
}

/// Menu bar state and rendering
pub struct MenuBar {
    /// Whether there's history to go back to
    can_go_back: bool,
    /// Whether there's history to go forward to
    can_go_forward: bool,
    /// Whether there's a tab to close
    has_tab: bool,
    /// Whether there are multiple tabs
    has_multiple_tabs: bool,
    /// Whether undo is available
    can_undo: bool,
    /// Whether redo is available
    can_redo: bool,
    /// Current zoom level (100 = normal)
    zoom_level: u32,
}

impl Default for MenuBar {
    fn default() -> Self {
        Self {
            can_go_back: false,
            can_go_forward: false,
            has_tab: true,
            has_multiple_tabs: false,
            can_undo: false,
            can_redo: false,
            zoom_level: 100,
        }
    }
}

impl MenuBar {
    /// Create a new menu bar
    pub fn new() -> Self {
        Self::default()
    }

    /// Update navigation state
    pub fn set_navigation_state(&mut self, can_go_back: bool, can_go_forward: bool) {
        self.can_go_back = can_go_back;
        self.can_go_forward = can_go_forward;
    }

    /// Update tab state
    pub fn set_tab_state(&mut self, has_tab: bool, has_multiple_tabs: bool) {
        self.has_tab = has_tab;
        self.has_multiple_tabs = has_multiple_tabs;
    }

    /// Update edit state
    pub fn set_edit_state(&mut self, can_undo: bool, can_redo: bool) {
        self.can_undo = can_undo;
        self.can_redo = can_redo;
    }

    /// Set zoom level
    pub fn set_zoom_level(&mut self, zoom: u32) {
        self.zoom_level = zoom;
    }

    /// Get current zoom level
    pub fn zoom_level(&self) -> u32 {
        self.zoom_level
    }

    /// Render the menu bar and return any triggered action
    pub fn render(&self, ui: &mut egui::Ui) -> Option<MenuAction> {
        let mut action = None;

        egui::menu::bar(ui, |ui| {
            // File menu
            ui.menu_button("File", |ui| {
                if ui.add(egui::Button::new("New Window").shortcut_text("Ctrl+N")).clicked() {
                    action = Some(MenuAction::UiAction(UiAction::About)); // Placeholder
                    ui.close_menu();
                }

                if ui.add(egui::Button::new("New Tab").shortcut_text("Ctrl+T")).clicked() {
                    action = Some(MenuAction::UiAction(UiAction::About)); // Placeholder
                    ui.close_menu();
                }

                if ui.add(egui::Button::new("New Private Window").shortcut_text("Ctrl+Shift+N")).clicked() {
                    action = Some(MenuAction::UiAction(UiAction::About)); // Placeholder
                    ui.close_menu();
                }

                ui.separator();

                if ui.add(egui::Button::new("Open File...").shortcut_text("Ctrl+O")).clicked() {
                    action = Some(MenuAction::UiAction(UiAction::About)); // Placeholder
                    ui.close_menu();
                }

                ui.separator();

                if ui.add_enabled(
                    self.has_tab,
                    egui::Button::new("Close Tab").shortcut_text("Ctrl+W")
                ).clicked() {
                    action = Some(MenuAction::UiAction(UiAction::About)); // Placeholder
                    ui.close_menu();
                }

                if ui.add_enabled(
                    true,
                    egui::Button::new("Close Window").shortcut_text("Ctrl+Shift+W")
                ).clicked() {
                    action = Some(MenuAction::UiAction(UiAction::About)); // Placeholder
                    ui.close_menu();
                }

                ui.separator();

                if ui.add(egui::Button::new("Exit").shortcut_text("Alt+F4")).clicked() {
                    action = Some(MenuAction::UiAction(UiAction::About)); // Placeholder
                    ui.close_menu();
                }
            });

            // Edit menu
            ui.menu_button("Edit", |ui| {
                if ui.add_enabled(
                    self.can_undo,
                    egui::Button::new("Undo").shortcut_text("Ctrl+Z")
                ).clicked() {
                    action = Some(MenuAction::UiAction(UiAction::Undo));
                    ui.close_menu();
                }

                if ui.add_enabled(
                    self.can_redo,
                    egui::Button::new("Redo").shortcut_text("Ctrl+Y")
                ).clicked() {
                    action = Some(MenuAction::UiAction(UiAction::Redo));
                    ui.close_menu();
                }

                ui.separator();

                if ui.add(egui::Button::new("Cut").shortcut_text("Ctrl+X")).clicked() {
                    action = Some(MenuAction::UiAction(UiAction::Cut));
                    ui.close_menu();
                }

                if ui.add(egui::Button::new("Copy").shortcut_text("Ctrl+C")).clicked() {
                    action = Some(MenuAction::UiAction(UiAction::Copy));
                    ui.close_menu();
                }

                if ui.add(egui::Button::new("Paste").shortcut_text("Ctrl+V")).clicked() {
                    action = Some(MenuAction::UiAction(UiAction::Paste));
                    ui.close_menu();
                }

                ui.separator();

                if ui.add(egui::Button::new("Select All").shortcut_text("Ctrl+A")).clicked() {
                    action = Some(MenuAction::UiAction(UiAction::SelectAll));
                    ui.close_menu();
                }

                ui.separator();

                if ui.add(egui::Button::new("Find...").shortcut_text("Ctrl+F")).clicked() {
                    action = Some(MenuAction::UiAction(UiAction::Find));
                    ui.close_menu();
                }
            });

            // View menu
            ui.menu_button("View", |ui| {
                if ui.add(egui::Button::new("Zoom In").shortcut_text("Ctrl++")).clicked() {
                    action = Some(MenuAction::UiAction(UiAction::ZoomIn));
                    ui.close_menu();
                }

                if ui.add(egui::Button::new("Zoom Out").shortcut_text("Ctrl+-")).clicked() {
                    action = Some(MenuAction::UiAction(UiAction::ZoomOut));
                    ui.close_menu();
                }

                if ui.add(egui::Button::new("Reset Zoom").shortcut_text("Ctrl+0")).clicked() {
                    action = Some(MenuAction::UiAction(UiAction::ResetZoom));
                    ui.close_menu();
                }

                ui.separator();

                ui.label(format!("Zoom: {}%", self.zoom_level));

                ui.separator();

                if ui.add(egui::Button::new("Full Screen").shortcut_text("F11")).clicked() {
                    action = Some(MenuAction::UiAction(UiAction::FullScreen));
                    ui.close_menu();
                }

                if ui.add(egui::Button::new("Developer Tools").shortcut_text("F12")).clicked() {
                    action = Some(MenuAction::TogglePanel(PanelType::DevTools));
                    ui.close_menu();
                }
            });

            // History menu
            ui.menu_button("History", |ui| {
                if ui.add_enabled(
                    self.can_go_back,
                    egui::Button::new("Back").shortcut_text("Alt+←")
                ).clicked() {
                    action = Some(MenuAction::UiAction(UiAction::About)); // Placeholder
                    ui.close_menu();
                }

                if ui.add_enabled(
                    self.can_go_forward,
                    egui::Button::new("Forward").shortcut_text("Alt+→")
                ).clicked() {
                    action = Some(MenuAction::UiAction(UiAction::About)); // Placeholder
                    ui.close_menu();
                }

                ui.separator();

                if ui.add(egui::Button::new("Show All History").shortcut_text("Ctrl+H")).clicked() {
                    action = Some(MenuAction::UiAction(UiAction::ShowAllHistory));
                    ui.close_menu();
                }

                if ui.add(egui::Button::new("Recently Closed Tabs")).clicked() {
                    action = Some(MenuAction::UiAction(UiAction::About)); // Placeholder
                    ui.close_menu();
                }

                ui.separator();

                if ui.add(egui::Button::new("Clear History...")).clicked() {
                    action = Some(MenuAction::UiAction(UiAction::ClearHistory));
                    ui.close_menu();
                }
            });

            // Bookmarks menu
            ui.menu_button("Bookmarks", |ui| {
                if ui.add(egui::Button::new("Bookmark This Page").shortcut_text("Ctrl+D")).clicked() {
                    action = Some(MenuAction::UiAction(UiAction::BookmarkPage));
                    ui.close_menu();
                }

                if ui.add(egui::Button::new("Show All Bookmarks").shortcut_text("Ctrl+Shift+B")).clicked() {
                    action = Some(MenuAction::UiAction(UiAction::ShowAllBookmarks));
                    ui.close_menu();
                }

                if ui.add(egui::Button::new("Bookmark Manager")).clicked() {
                    action = Some(MenuAction::TogglePanel(PanelType::Bookmarks));
                    ui.close_menu();
                }
            });

            // Tools menu
            ui.menu_button("Tools", |ui| {
                if ui.add(egui::Button::new("Downloads").shortcut_text("Ctrl+J")).clicked() {
                    action = Some(MenuAction::TogglePanel(PanelType::Downloads));
                    ui.close_menu();
                }

                if ui.add(egui::Button::new("Extensions")).clicked() {
                    action = Some(MenuAction::UiAction(UiAction::About)); // Placeholder
                    ui.close_menu();
                }

                if ui.add(egui::Button::new("Developer Tools").shortcut_text("F12")).clicked() {
                    action = Some(MenuAction::TogglePanel(PanelType::DevTools));
                    ui.close_menu();
                }

                ui.separator();

                if ui.add(egui::Button::new("Settings").shortcut_text("Ctrl+,")).clicked() {
                    action = Some(MenuAction::TogglePanel(PanelType::Settings));
                    ui.close_menu();
                }
            });

            // Help menu
            ui.menu_button("Help", |ui| {
                if ui.add(egui::Button::new("About")).clicked() {
                    action = Some(MenuAction::UiAction(UiAction::About));
                    ui.close_menu();
                }

                if ui.add(egui::Button::new("Documentation")).clicked() {
                    action = Some(MenuAction::UiAction(UiAction::ShowDocumentation));
                    ui.close_menu();
                }

                if ui.add(egui::Button::new("Report Issue")).clicked() {
                    action = Some(MenuAction::UiAction(UiAction::ReportIssue));
                    ui.close_menu();
                }
            });
        });

        action
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_bar_default() {
        let menu = MenuBar::new();
        assert!(!menu.can_go_back);
        assert!(!menu.can_go_forward);
        assert!(menu.has_tab);
        assert!(!menu.has_multiple_tabs);
        assert_eq!(menu.zoom_level, 100);
    }

    #[test]
    fn test_set_navigation_state() {
        let mut menu = MenuBar::new();
        menu.set_navigation_state(true, false);
        assert!(menu.can_go_back);
        assert!(!menu.can_go_forward);
    }

    #[test]
    fn test_set_tab_state() {
        let mut menu = MenuBar::new();
        menu.set_tab_state(true, true);
        assert!(menu.has_tab);
        assert!(menu.has_multiple_tabs);
    }

    #[test]
    fn test_set_edit_state() {
        let mut menu = MenuBar::new();
        menu.set_edit_state(true, true);
        assert!(menu.can_undo);
        assert!(menu.can_redo);
    }

    #[test]
    fn test_set_zoom_level() {
        let mut menu = MenuBar::new();
        menu.set_zoom_level(150);
        assert_eq!(menu.zoom_level, 150);
    }
}
