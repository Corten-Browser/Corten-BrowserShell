//! Crash Recovery UI
//!
//! Provides UI elements for crash recovery including:
//! - Session restore dialog
//! - Recently closed tabs tracking and restoration

use egui::{Context, Ui};
use serde::{Deserialize, Serialize};
use shared_types::TabId;
use std::collections::VecDeque;

/// Information about a recently closed tab
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClosedTabInfo {
    /// Original tab ID
    pub id: TabId,

    /// Tab title
    pub title: String,

    /// URL that was loaded
    pub url: String,

    /// Timestamp when closed (seconds since UNIX epoch)
    pub closed_at: u64,
}

/// Session restore dialog state
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SessionRestoreDialog {
    /// Dialog is hidden
    Hidden,

    /// Dialog is visible
    Visible,

    /// User chose to restore
    RestoreChosen,

    /// User chose not to restore
    DismissChosen,
}

impl Default for SessionRestoreDialog {
    fn default() -> Self {
        Self::Hidden
    }
}

/// Recently closed tabs tracker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentlyClosedTabs {
    /// List of closed tabs (most recent first)
    tabs: VecDeque<ClosedTabInfo>,

    /// Maximum number of closed tabs to track
    max_tabs: usize,
}

impl Default for RecentlyClosedTabs {
    fn default() -> Self {
        Self {
            tabs: VecDeque::new(),
            max_tabs: 10, // Track last 10 closed tabs
        }
    }
}

impl RecentlyClosedTabs {
    /// Create a new recently closed tabs tracker
    pub fn new(max_tabs: usize) -> Self {
        Self {
            tabs: VecDeque::new(),
            max_tabs,
        }
    }

    /// Add a closed tab to the tracker
    pub fn add(&mut self, tab: ClosedTabInfo) {
        // Add to front (most recent)
        self.tabs.push_front(tab);

        // Trim to max size
        while self.tabs.len() > self.max_tabs {
            self.tabs.pop_back();
        }
    }

    /// Get all closed tabs (most recent first)
    pub fn get_all(&self) -> &VecDeque<ClosedTabInfo> {
        &self.tabs
    }

    /// Remove a tab from the tracker (when it's restored)
    pub fn remove(&mut self, tab_id: TabId) -> Option<ClosedTabInfo> {
        if let Some(index) = self.tabs.iter().position(|t| t.id == tab_id) {
            self.tabs.remove(index)
        } else {
            None
        }
    }

    /// Clear all closed tabs
    pub fn clear(&mut self) {
        self.tabs.clear();
    }

    /// Check if there are any closed tabs
    pub fn is_empty(&self) -> bool {
        self.tabs.is_empty()
    }

    /// Get the count of closed tabs
    pub fn count(&self) -> usize {
        self.tabs.len()
    }
}

/// Crash recovery UI manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrashRecoveryUi {
    /// Session restore dialog state
    pub restore_dialog: SessionRestoreDialog,

    /// Recently closed tabs tracker
    pub recently_closed: RecentlyClosedTabs,

    /// Whether to show recently closed tabs menu
    pub show_recently_closed_menu: bool,
}

impl Default for CrashRecoveryUi {
    fn default() -> Self {
        Self {
            restore_dialog: SessionRestoreDialog::Hidden,
            recently_closed: RecentlyClosedTabs::default(),
            show_recently_closed_menu: false,
        }
    }
}

impl CrashRecoveryUi {
    /// Create a new crash recovery UI manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Show the session restore dialog
    pub fn show_restore_dialog(&mut self) {
        self.restore_dialog = SessionRestoreDialog::Visible;
    }

    /// Check if user chose to restore session
    pub fn should_restore_session(&self) -> bool {
        self.restore_dialog == SessionRestoreDialog::RestoreChosen
    }

    /// Check if user dismissed the restore dialog
    pub fn restore_dialog_dismissed(&self) -> bool {
        self.restore_dialog == SessionRestoreDialog::DismissChosen
    }

    /// Reset the restore dialog (after handling the choice)
    pub fn reset_restore_dialog(&mut self) {
        self.restore_dialog = SessionRestoreDialog::Hidden;
    }

    /// Render the session restore dialog
    ///
    /// Returns true if the user made a choice (restore or dismiss)
    pub fn render_restore_dialog(&mut self, ctx: &Context) -> bool {
        if self.restore_dialog != SessionRestoreDialog::Visible {
            return false;
        }

        let mut choice_made = false;

        egui::Window::new("⚠ Corten Browser Crashed")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);

                    ui.label(egui::RichText::new(
                        "The browser closed unexpectedly."
                    ).size(16.0));

                    ui.add_space(10.0);

                    ui.label("Would you like to restore your previous session?");

                    ui.add_space(20.0);

                    ui.horizontal(|ui| {
                        if ui.button(egui::RichText::new("🔄 Restore Session").size(14.0)).clicked() {
                            self.restore_dialog = SessionRestoreDialog::RestoreChosen;
                            choice_made = true;
                        }

                        ui.add_space(10.0);

                        if ui.button(egui::RichText::new("✕ Start Fresh").size(14.0)).clicked() {
                            self.restore_dialog = SessionRestoreDialog::DismissChosen;
                            choice_made = true;
                        }
                    });

                    ui.add_space(10.0);
                });
            });

        choice_made
    }

    /// Render recently closed tabs menu
    ///
    /// Returns the ID of a tab to restore, if user clicked one
    pub fn render_recently_closed_menu(&mut self, ui: &mut Ui) -> Option<TabId> {
        if !self.show_recently_closed_menu {
            return None;
        }

        let mut tab_to_restore: Option<TabId> = None;
        let mut should_close = false;

        egui::Window::new("Recently Closed Tabs")
            .default_width(400.0)
            .resizable(true)
            .open(&mut self.show_recently_closed_menu)
            .show(ui.ctx(), |ui| {
                ui.add_space(5.0);

                if self.recently_closed.is_empty() {
                    ui.vertical_centered(|ui| {
                        ui.label("No recently closed tabs");
                    });
                } else {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for closed_tab in self.recently_closed.get_all() {
                            ui.group(|ui| {
                                ui.horizontal(|ui| {
                                    ui.vertical(|ui| {
                                        ui.strong(&closed_tab.title);
                                        ui.label(
                                            egui::RichText::new(&closed_tab.url)
                                                .small()
                                                .color(ui.style().visuals.weak_text_color()),
                                        );
                                    });

                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            if ui.small_button("🔄 Restore").clicked() {
                                                tab_to_restore = Some(closed_tab.id);
                                            }
                                        },
                                    );
                                });
                            });

                            ui.add_space(5.0);
                        }
                    });
                }

                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Clear All").clicked() && !self.recently_closed.is_empty() {
                        self.recently_closed.clear();
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Close").clicked() {
                            should_close = true;
                        }
                    });
                });
            });

        if should_close {
            self.show_recently_closed_menu = false;
        }

        tab_to_restore
    }

    /// Toggle the recently closed tabs menu
    pub fn toggle_recently_closed_menu(&mut self) {
        self.show_recently_closed_menu = !self.show_recently_closed_menu;
    }

    /// Add a closed tab to the recently closed list
    pub fn add_closed_tab(&mut self, tab: ClosedTabInfo) {
        self.recently_closed.add(tab);
    }

    /// Remove a tab from recently closed (when restored)
    pub fn remove_closed_tab(&mut self, tab_id: TabId) -> Option<ClosedTabInfo> {
        self.recently_closed.remove(tab_id)
    }

    /// Get count of recently closed tabs
    pub fn closed_tab_count(&self) -> usize {
        self.recently_closed.count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recently_closed_tabs_add() {
        let mut tracker = RecentlyClosedTabs::new(5);

        let tab = ClosedTabInfo {
            id: TabId::new(),
            title: "Test Tab".to_string(),
            url: "https://example.com".to_string(),
            closed_at: 1234567890,
        };

        tracker.add(tab.clone());

        assert_eq!(tracker.count(), 1);
        assert_eq!(tracker.get_all()[0].title, "Test Tab");
    }

    #[test]
    fn test_recently_closed_tabs_max_limit() {
        let mut tracker = RecentlyClosedTabs::new(3);

        for i in 0..5 {
            let tab = ClosedTabInfo {
                id: TabId::new(),
                title: format!("Tab {}", i),
                url: format!("https://example{}.com", i),
                closed_at: 1234567890 + i as u64,
            };
            tracker.add(tab);
        }

        assert_eq!(tracker.count(), 3);
        // Most recent should be Tab 4
        assert_eq!(tracker.get_all()[0].title, "Tab 4");
        // Oldest should be Tab 2 (Tab 0 and 1 were dropped)
        assert_eq!(tracker.get_all()[2].title, "Tab 2");
    }

    #[test]
    fn test_recently_closed_tabs_remove() {
        let mut tracker = RecentlyClosedTabs::new(5);

        let tab_id = TabId::new();
        let tab = ClosedTabInfo {
            id: tab_id,
            title: "Test Tab".to_string(),
            url: "https://example.com".to_string(),
            closed_at: 1234567890,
        };

        tracker.add(tab);
        assert_eq!(tracker.count(), 1);

        let removed = tracker.remove(tab_id);
        assert!(removed.is_some());
        assert_eq!(tracker.count(), 0);
    }

    #[test]
    fn test_crash_recovery_ui_default() {
        let ui = CrashRecoveryUi::default();
        assert_eq!(ui.restore_dialog, SessionRestoreDialog::Hidden);
        assert!(ui.recently_closed.is_empty());
        assert!(!ui.show_recently_closed_menu);
    }

    #[test]
    fn test_crash_recovery_ui_show_dialog() {
        let mut ui = CrashRecoveryUi::new();
        ui.show_restore_dialog();

        assert_eq!(ui.restore_dialog, SessionRestoreDialog::Visible);
        assert!(!ui.should_restore_session());
        assert!(!ui.restore_dialog_dismissed());
    }

    #[test]
    fn test_crash_recovery_ui_restore_chosen() {
        let mut ui = CrashRecoveryUi::new();
        ui.restore_dialog = SessionRestoreDialog::RestoreChosen;

        assert!(ui.should_restore_session());
        assert!(!ui.restore_dialog_dismissed());
    }

    #[test]
    fn test_crash_recovery_ui_dismiss_chosen() {
        let mut ui = CrashRecoveryUi::new();
        ui.restore_dialog = SessionRestoreDialog::DismissChosen;

        assert!(!ui.should_restore_session());
        assert!(ui.restore_dialog_dismissed());
    }

    #[test]
    fn test_crash_recovery_ui_add_closed_tab() {
        let mut ui = CrashRecoveryUi::new();

        let tab = ClosedTabInfo {
            id: TabId::new(),
            title: "Test Tab".to_string(),
            url: "https://example.com".to_string(),
            closed_at: 1234567890,
        };

        ui.add_closed_tab(tab);

        assert_eq!(ui.closed_tab_count(), 1);
    }
}
