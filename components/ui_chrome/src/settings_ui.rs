//! Settings UI Panel
//!
//! Provides a tabbed settings interface with the following categories:
//! - General: Home page, startup behavior, default search engine
//! - Appearance: Theme, font size, toolbar customization
//! - Privacy: Clear browsing data, cookie policy, tracking protection, DNT
//! - Security: Safe browsing, password manager settings
//! - Downloads: Default download location, ask where to save
//! - Advanced: Hardware acceleration, proxy settings, reset settings

use egui::{Context, ScrollArea, Ui};
use serde::{Deserialize, Serialize};
use settings_manager::SettingValue;
use std::collections::HashMap;

/// Settings tab selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SettingsTab {
    General,
    Appearance,
    Privacy,
    Security,
    Downloads,
    Advanced,
}

impl SettingsTab {
    /// Get all tabs in order
    pub fn all() -> Vec<Self> {
        vec![
            Self::General,
            Self::Appearance,
            Self::Privacy,
            Self::Security,
            Self::Downloads,
            Self::Advanced,
        ]
    }

    /// Get tab display name
    pub fn name(&self) -> &str {
        match self {
            Self::General => "General",
            Self::Appearance => "Appearance",
            Self::Privacy => "Privacy",
            Self::Security => "Security",
            Self::Downloads => "Downloads",
            Self::Advanced => "Advanced",
        }
    }

    /// Get tab icon
    pub fn icon(&self) -> &str {
        match self {
            Self::General => "⚙",
            Self::Appearance => "🎨",
            Self::Privacy => "🔒",
            Self::Security => "🛡",
            Self::Downloads => "⬇",
            Self::Advanced => "🔧",
        }
    }
}

/// Settings UI state and configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsUi {
    /// Currently selected tab
    pub current_tab: SettingsTab,

    /// Local cache of settings (for UI responsiveness)
    /// Maps setting key to value as string representation
    #[serde(skip)]
    pub settings_cache: HashMap<String, SettingValue>,

    /// Whether settings have been modified and not saved
    #[serde(skip)]
    pub has_unsaved_changes: bool,
}

impl Default for SettingsUi {
    fn default() -> Self {
        Self {
            current_tab: SettingsTab::General,
            settings_cache: HashMap::new(),
            has_unsaved_changes: false,
        }
    }
}

impl SettingsUi {
    /// Create a new SettingsUi
    pub fn new() -> Self {
        Self::default()
    }

    /// Load settings from a HashMap (typically from settings_manager)
    pub fn load_settings(&mut self, settings: HashMap<String, SettingValue>) {
        self.settings_cache = settings;
        self.has_unsaved_changes = false;
    }

    /// Get all settings from the cache (for saving to settings_manager)
    pub fn get_all_settings(&self) -> &HashMap<String, SettingValue> {
        &self.settings_cache
    }

    /// Clear the unsaved changes flag (after successful save)
    pub fn mark_saved(&mut self) {
        self.has_unsaved_changes = false;
    }

    /// Update a setting value in the local cache
    pub fn update_setting(&mut self, key: String, value: SettingValue) {
        self.settings_cache.insert(key, value);
        self.has_unsaved_changes = true;
    }

    /// Get a setting value from the local cache
    pub fn get_setting(&self, key: &str) -> Option<&SettingValue> {
        self.settings_cache.get(key)
    }

    /// Get a string setting value, with fallback
    pub fn get_string(&self, key: &str, default: &str) -> String {
        match self.get_setting(key) {
            Some(SettingValue::String(s)) => s.clone(),
            _ => default.to_string(),
        }
    }

    /// Get a boolean setting value, with fallback
    pub fn get_bool(&self, key: &str, default: bool) -> bool {
        match self.get_setting(key) {
            Some(SettingValue::Boolean(b)) => *b,
            _ => default,
        }
    }

    /// Get an integer setting value, with fallback
    pub fn get_int(&self, key: &str, default: i64) -> i64 {
        match self.get_setting(key) {
            Some(SettingValue::Integer(i)) => *i,
            _ => default,
        }
    }

    /// Render the settings panel
    pub fn show(&mut self, ctx: &Context) {
        egui::SidePanel::left("settings_tabs")
            .default_width(150.0)
            .min_width(120.0)
            .show(ctx, |ui| {
                self.render_tab_list(ui);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_current_tab(ui);
        });
    }

    /// Render the tab list on the left side
    fn render_tab_list(&mut self, ui: &mut Ui) {
        ui.heading("Settings");
        ui.separator();

        for tab in SettingsTab::all() {
            let is_selected = self.current_tab == tab;
            let label = format!("{} {}", tab.icon(), tab.name());

            if ui.selectable_label(is_selected, label).clicked() {
                self.current_tab = tab;
            }
        }

        ui.separator();

        // Save/Reset buttons at the bottom
        ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
            if self.has_unsaved_changes {
                ui.label(egui::RichText::new("⚠ Unsaved changes").color(egui::Color32::YELLOW));

                ui.horizontal(|ui| {
                    if ui.button("💾 Save").clicked() {
                        // TODO: Save settings via settings_manager
                        self.has_unsaved_changes = false;
                    }

                    if ui.button("↶ Discard").clicked() {
                        // TODO: Reload settings from settings_manager
                        self.has_unsaved_changes = false;
                    }
                });
            }
        });
    }

    /// Render the currently selected tab content
    fn render_current_tab(&mut self, ui: &mut Ui) {
        ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(10.0);

            match self.current_tab {
                SettingsTab::General => self.render_general_tab(ui),
                SettingsTab::Appearance => self.render_appearance_tab(ui),
                SettingsTab::Privacy => self.render_privacy_tab(ui),
                SettingsTab::Security => self.render_security_tab(ui),
                SettingsTab::Downloads => self.render_downloads_tab(ui),
                SettingsTab::Advanced => self.render_advanced_tab(ui),
            }

            ui.add_space(20.0);
        });
    }

    /// Render General settings tab
    fn render_general_tab(&mut self, ui: &mut Ui) {
        ui.heading("⚙ General Settings");
        ui.separator();
        ui.add_space(10.0);

        // Home page
        ui.label(egui::RichText::new("Home Page").strong());
        let mut home_page = self.get_string("general.home_page", "about:blank");
        if ui.text_edit_singleline(&mut home_page).changed() {
            self.update_setting("general.home_page".to_string(), SettingValue::String(home_page));
        }
        ui.label("The page to load when opening a new tab");
        ui.add_space(10.0);

        // Startup behavior
        ui.label(egui::RichText::new("On Startup").strong());
        let mut startup = self.get_string("general.startup_behavior", "new_tab");
        egui::ComboBox::from_id_salt("startup_behavior")
            .selected_text(match startup.as_str() {
                "new_tab" => "Open New Tab",
                "home_page" => "Open Home Page",
                "restore_session" => "Restore Last Session",
                _ => "Open New Tab",
            })
            .show_ui(ui, |ui| {
                if ui.selectable_value(&mut startup, "new_tab".to_string(), "Open New Tab").changed() {
                    self.update_setting("general.startup_behavior".to_string(), SettingValue::String(startup.clone()));
                }
                if ui.selectable_value(&mut startup, "home_page".to_string(), "Open Home Page").changed() {
                    self.update_setting("general.startup_behavior".to_string(), SettingValue::String(startup.clone()));
                }
                if ui.selectable_value(&mut startup, "restore_session".to_string(), "Restore Last Session").changed() {
                    self.update_setting("general.startup_behavior".to_string(), SettingValue::String(startup.clone()));
                }
            });
        ui.add_space(10.0);

        // Default search engine
        ui.label(egui::RichText::new("Default Search Engine").strong());
        let mut search_engine = self.get_string("general.default_search_engine", "DuckDuckGo");
        egui::ComboBox::from_id_salt("search_engine")
            .selected_text(&search_engine)
            .show_ui(ui, |ui| {
                for engine in &["DuckDuckGo", "Google", "Bing", "Brave Search", "Ecosia"] {
                    if ui.selectable_value(&mut search_engine, engine.to_string(), *engine).changed() {
                        self.update_setting("general.default_search_engine".to_string(), SettingValue::String(search_engine.clone()));
                    }
                }
            });
    }

    /// Render Appearance settings tab
    fn render_appearance_tab(&mut self, ui: &mut Ui) {
        ui.heading("🎨 Appearance Settings");
        ui.separator();
        ui.add_space(10.0);

        // Theme
        ui.label(egui::RichText::new("Theme").strong());
        let mut theme = self.get_string("ui.theme", "light");
        egui::ComboBox::from_id_salt("theme")
            .selected_text(match theme.as_str() {
                "light" => "Light",
                "dark" => "Dark",
                "auto" => "Auto (System)",
                _ => "Light",
            })
            .show_ui(ui, |ui| {
                if ui.selectable_value(&mut theme, "light".to_string(), "Light").changed() {
                    self.update_setting("ui.theme".to_string(), SettingValue::String(theme.clone()));
                }
                if ui.selectable_value(&mut theme, "dark".to_string(), "Dark").changed() {
                    self.update_setting("ui.theme".to_string(), SettingValue::String(theme.clone()));
                }
                if ui.selectable_value(&mut theme, "auto".to_string(), "Auto (System)").changed() {
                    self.update_setting("ui.theme".to_string(), SettingValue::String(theme.clone()));
                }
            });
        ui.label("Select the color theme for the browser");
        ui.add_space(10.0);

        // Font size
        ui.label(egui::RichText::new("Font Size").strong());
        let mut font_size = self.get_int("ui.font_size", 14);
        if ui.add(egui::Slider::new(&mut font_size, 10..=24).suffix(" px")).changed() {
            self.update_setting("ui.font_size".to_string(), SettingValue::Integer(font_size));
        }
        ui.label("Default font size for browser UI");
        ui.add_space(10.0);

        // Toolbar customization
        ui.label(egui::RichText::new("Toolbar Style").strong());
        let mut toolbar = self.get_string("appearance.toolbar_customization", "default");
        egui::ComboBox::from_id_salt("toolbar")
            .selected_text(match toolbar.as_str() {
                "default" => "Default",
                "compact" => "Compact",
                "minimal" => "Minimal",
                _ => "Default",
            })
            .show_ui(ui, |ui| {
                if ui.selectable_value(&mut toolbar, "default".to_string(), "Default").changed() {
                    self.update_setting("appearance.toolbar_customization".to_string(), SettingValue::String(toolbar.clone()));
                }
                if ui.selectable_value(&mut toolbar, "compact".to_string(), "Compact").changed() {
                    self.update_setting("appearance.toolbar_customization".to_string(), SettingValue::String(toolbar.clone()));
                }
                if ui.selectable_value(&mut toolbar, "minimal".to_string(), "Minimal").changed() {
                    self.update_setting("appearance.toolbar_customization".to_string(), SettingValue::String(toolbar.clone()));
                }
            });
        ui.label("Customize the toolbar appearance");
        ui.add_space(10.0);

        // Show bookmarks bar
        let mut show_bookmarks = self.get_bool("ui.show_bookmarks_bar", true);
        if ui.checkbox(&mut show_bookmarks, "Show bookmarks bar").changed() {
            self.update_setting("ui.show_bookmarks_bar".to_string(), SettingValue::Boolean(show_bookmarks));
        }

        // Animations
        let mut animations = self.get_bool("ui.animations_enabled", true);
        if ui.checkbox(&mut animations, "Enable animations").changed() {
            self.update_setting("ui.animations_enabled".to_string(), SettingValue::Boolean(animations));
        }
    }

    /// Render Privacy settings tab
    fn render_privacy_tab(&mut self, ui: &mut Ui) {
        ui.heading("🔒 Privacy Settings");
        ui.separator();
        ui.add_space(10.0);

        // Do Not Track
        let mut dnt = self.get_bool("privacy.do_not_track", true);
        if ui.checkbox(&mut dnt, "Send \"Do Not Track\" request").changed() {
            self.update_setting("privacy.do_not_track".to_string(), SettingValue::Boolean(dnt));
        }
        ui.label("Request that websites not track your browsing activity");
        ui.add_space(10.0);

        // Tracking protection
        let mut tracking = self.get_bool("privacy.tracking_protection", true);
        if ui.checkbox(&mut tracking, "Enable tracking protection").changed() {
            self.update_setting("privacy.tracking_protection".to_string(), SettingValue::Boolean(tracking));
        }
        ui.label("Block known trackers and third-party cookies");
        ui.add_space(10.0);

        // Cookie policy
        ui.label(egui::RichText::new("Cookie Policy").strong());
        let mut cookies = self.get_string("privacy.cookie_policy", "allow_all");
        egui::ComboBox::from_id_salt("cookies")
            .selected_text(match cookies.as_str() {
                "allow_all" => "Allow All Cookies",
                "block_third_party" => "Block Third-Party Cookies",
                "block_all" => "Block All Cookies",
                _ => "Allow All Cookies",
            })
            .show_ui(ui, |ui| {
                if ui.selectable_value(&mut cookies, "allow_all".to_string(), "Allow All Cookies").changed() {
                    self.update_setting("privacy.cookie_policy".to_string(), SettingValue::String(cookies.clone()));
                }
                if ui.selectable_value(&mut cookies, "block_third_party".to_string(), "Block Third-Party Cookies").changed() {
                    self.update_setting("privacy.cookie_policy".to_string(), SettingValue::String(cookies.clone()));
                }
                if ui.selectable_value(&mut cookies, "block_all".to_string(), "Block All Cookies").changed() {
                    self.update_setting("privacy.cookie_policy".to_string(), SettingValue::String(cookies.clone()));
                }
            });
        ui.add_space(10.0);

        // Clear on exit
        let mut clear_on_exit = self.get_bool("privacy.clear_on_exit", false);
        if ui.checkbox(&mut clear_on_exit, "Clear browsing data on exit").changed() {
            self.update_setting("privacy.clear_on_exit".to_string(), SettingValue::Boolean(clear_on_exit));
        }
        ui.label("Clear cookies and site data when closing the browser");
        ui.add_space(20.0);

        // Clear browsing data button
        ui.separator();
        if ui.button("🗑 Clear Browsing Data...").clicked() {
            // TODO: Show clear browsing data dialog
        }

        ui.add_space(20.0);
        ui.separator();
        ui.add_space(10.0);

        // Ad Blocker section
        ui.heading("🛡 Ad Blocker");
        ui.add_space(10.0);

        // Enable ad blocker
        let mut ad_blocker_enabled = self.get_bool("privacy.ad_blocker_enabled", true);
        if ui.checkbox(&mut ad_blocker_enabled, "Enable ad blocker").changed() {
            self.update_setting("privacy.ad_blocker_enabled".to_string(), SettingValue::Boolean(ad_blocker_enabled));
        }
        ui.label("Block ads, trackers, and malicious content");
        ui.add_space(10.0);

        if ad_blocker_enabled {
            // Block malware domains
            let mut block_malware = self.get_bool("privacy.block_malware_domains", true);
            if ui.checkbox(&mut block_malware, "Block malware domains").changed() {
                self.update_setting("privacy.block_malware_domains".to_string(), SettingValue::Boolean(block_malware));
            }
            ui.label("Protect against known malicious websites");
            ui.add_space(10.0);

            // Block tracking scripts
            let mut block_tracking = self.get_bool("privacy.block_tracking_scripts", true);
            if ui.checkbox(&mut block_tracking, "Block tracking scripts").changed() {
                self.update_setting("privacy.block_tracking_scripts".to_string(), SettingValue::Boolean(block_tracking));
            }
            ui.label("Prevent websites from tracking your activity");
            ui.add_space(10.0);

            // Hide elements
            let mut hide_elements = self.get_bool("privacy.hide_blocked_elements", true);
            if ui.checkbox(&mut hide_elements, "Hide blocked elements").changed() {
                self.update_setting("privacy.hide_blocked_elements".to_string(), SettingValue::Boolean(hide_elements));
            }
            ui.label("Remove blocked ad elements from pages");
            ui.add_space(10.0);

            // Aggressive blocking
            let mut aggressive = self.get_bool("privacy.aggressive_blocking", false);
            if ui.checkbox(&mut aggressive, "Aggressive blocking mode").changed() {
                self.update_setting("privacy.aggressive_blocking".to_string(), SettingValue::Boolean(aggressive));
            }
            ui.label("Block more aggressively (may break some websites)");
            ui.add_space(10.0);

            // Filter list selection
            ui.label(egui::RichText::new("Filter Lists").strong());
            let mut filter_lists = self.get_string("privacy.filter_lists", "easylist");
            ui.horizontal(|ui| {
                let mut use_easylist = filter_lists.contains("easylist");
                if ui.checkbox(&mut use_easylist, "EasyList").changed() {
                    if use_easylist {
                        filter_lists.push_str(",easylist");
                    } else {
                        filter_lists = filter_lists.replace("easylist", "").replace(",,", ",");
                    }
                    self.update_setting("privacy.filter_lists".to_string(), SettingValue::String(filter_lists.clone()));
                }

                let mut use_easyprivacy = filter_lists.contains("easyprivacy");
                if ui.checkbox(&mut use_easyprivacy, "EasyPrivacy").changed() {
                    if use_easyprivacy {
                        filter_lists.push_str(",easyprivacy");
                    } else {
                        filter_lists = filter_lists.replace("easyprivacy", "").replace(",,", ",");
                    }
                    self.update_setting("privacy.filter_lists".to_string(), SettingValue::String(filter_lists.clone()));
                }

                let mut use_fanboy = filter_lists.contains("fanboy");
                if ui.checkbox(&mut use_fanboy, "Fanboy Annoyances").changed() {
                    if use_fanboy {
                        filter_lists.push_str(",fanboy");
                    } else {
                        filter_lists = filter_lists.replace("fanboy", "").replace(",,", ",");
                    }
                    self.update_setting("privacy.filter_lists".to_string(), SettingValue::String(filter_lists.clone()));
                }
            });
            ui.label("Choose which filter lists to use for blocking");
            ui.add_space(20.0);

            // Whitelist management
            ui.separator();
            if ui.button("📋 Manage Whitelist...").clicked() {
                // TODO: Show whitelist management dialog
            }
            ui.label("Manage sites that bypass ad blocking");
        }
    }

    /// Render Security settings tab
    fn render_security_tab(&mut self, ui: &mut Ui) {
        ui.heading("🛡 Security Settings");
        ui.separator();
        ui.add_space(10.0);

        // Safe browsing
        let mut safe_browsing = self.get_bool("security.safe_browsing", true);
        if ui.checkbox(&mut safe_browsing, "Enable safe browsing").changed() {
            self.update_setting("security.safe_browsing".to_string(), SettingValue::Boolean(safe_browsing));
        }
        ui.label("Protect against dangerous sites and downloads");
        ui.add_space(10.0);

        // Sandbox
        let mut sandbox = self.get_bool("security.enable_sandbox", true);
        if ui.checkbox(&mut sandbox, "Enable sandboxing").changed() {
            self.update_setting("security.enable_sandbox".to_string(), SettingValue::Boolean(sandbox));
        }
        ui.label("Isolate web content in a security sandbox");
        ui.add_space(10.0);

        // JavaScript
        let mut javascript = self.get_bool("security.allow_javascript", true);
        if ui.checkbox(&mut javascript, "Allow JavaScript").changed() {
            self.update_setting("security.allow_javascript".to_string(), SettingValue::Boolean(javascript));
        }
        ui.label("Enable JavaScript on all websites (some sites may not work if disabled)");
        ui.add_space(10.0);

        // Third-party cookies
        let mut block_cookies = self.get_bool("security.block_third_party_cookies", true);
        if ui.checkbox(&mut block_cookies, "Block third-party cookies").changed() {
            self.update_setting("security.block_third_party_cookies".to_string(), SettingValue::Boolean(block_cookies));
        }
        ui.add_space(10.0);

        // Password manager
        let mut password_manager = self.get_bool("security.password_manager_enabled", true);
        if ui.checkbox(&mut password_manager, "Enable password manager").changed() {
            self.update_setting("security.password_manager_enabled".to_string(), SettingValue::Boolean(password_manager));
        }
        ui.label("Offer to save and auto-fill passwords");
        ui.add_space(20.0);

        ui.separator();
        if password_manager {
            if ui.button("🔑 Manage Saved Passwords...").clicked() {
                // TODO: Show password manager dialog
            }
        }
    }

    /// Render Downloads settings tab
    fn render_downloads_tab(&mut self, ui: &mut Ui) {
        ui.heading("⬇ Downloads Settings");
        ui.separator();
        ui.add_space(10.0);

        // Default download location
        ui.label(egui::RichText::new("Download Location").strong());
        let mut download_path = self.get_string("downloads.default_location", "~/Downloads");
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut download_path);
            if ui.button("📁 Browse...").clicked() {
                // TODO: Show folder picker dialog
            }
        });
        if ui.button("↻ Use Default").clicked() {
            download_path = "~/Downloads".to_string();
            self.update_setting("downloads.default_location".to_string(), SettingValue::String(download_path.clone()));
        }
        ui.add_space(10.0);

        // Ask where to save
        let mut ask_location = self.get_bool("downloads.ask_where_to_save", false);
        if ui.checkbox(&mut ask_location, "Ask where to save each file before downloading").changed() {
            self.update_setting("downloads.ask_where_to_save".to_string(), SettingValue::Boolean(ask_location));
        }
        ui.add_space(20.0);

        ui.separator();
        if ui.button("📂 Open Downloads Folder").clicked() {
            // TODO: Open downloads folder in system file manager
        }
    }

    /// Render Advanced settings tab
    fn render_advanced_tab(&mut self, ui: &mut Ui) {
        ui.heading("🔧 Advanced Settings");
        ui.separator();
        ui.add_space(10.0);

        // Hardware acceleration
        let mut hw_accel = self.get_bool("advanced.hardware_acceleration", true);
        if ui.checkbox(&mut hw_accel, "Use hardware acceleration when available").changed() {
            self.update_setting("advanced.hardware_acceleration".to_string(), SettingValue::Boolean(hw_accel));
        }
        ui.label("Improves performance but may cause issues on some systems");
        ui.add_space(10.0);

        // HTTP/2
        let mut http2 = self.get_bool("network.enable_http2", true);
        if ui.checkbox(&mut http2, "Enable HTTP/2").changed() {
            self.update_setting("network.enable_http2".to_string(), SettingValue::Boolean(http2));
        }
        ui.add_space(10.0);

        // HTTP/3 / QUIC
        let mut http3 = self.get_bool("network.enable_http3", true);
        if ui.checkbox(&mut http3, "Enable HTTP/3 (QUIC)").changed() {
            self.update_setting("network.enable_http3".to_string(), SettingValue::Boolean(http3));
        }
        ui.add_space(20.0);

        // Proxy settings
        ui.separator();
        ui.label(egui::RichText::new("Proxy Settings").strong());

        let mut proxy_enabled = self.get_bool("advanced.proxy_enabled", false);
        if ui.checkbox(&mut proxy_enabled, "Use proxy server").changed() {
            self.update_setting("advanced.proxy_enabled".to_string(), SettingValue::Boolean(proxy_enabled));
        }

        if proxy_enabled {
            ui.add_space(5.0);
            ui.label("Proxy Host:");
            let mut proxy_host = self.get_string("advanced.proxy_host", "");
            if ui.text_edit_singleline(&mut proxy_host).changed() {
                self.update_setting("advanced.proxy_host".to_string(), SettingValue::String(proxy_host));
            }

            ui.label("Proxy Port:");
            let mut proxy_port = self.get_int("advanced.proxy_port", 8080);
            if ui.add(egui::DragValue::new(&mut proxy_port).range(1..=65535)).changed() {
                self.update_setting("advanced.proxy_port".to_string(), SettingValue::Integer(proxy_port));
            }
        }
        ui.add_space(20.0);

        // Reset settings
        ui.separator();
        ui.label(egui::RichText::new("Reset").strong());
        if ui.button("⚠ Reset All Settings to Defaults").clicked() {
            // TODO: Show confirmation dialog and reset settings
        }
        ui.label("This will reset all settings to their default values");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_ui_creation() {
        let ui = SettingsUi::new();
        assert_eq!(ui.current_tab, SettingsTab::General);
        assert!(!ui.has_unsaved_changes);
    }

    #[test]
    fn test_setting_update() {
        let mut ui = SettingsUi::new();
        ui.update_setting("test.key".to_string(), SettingValue::String("value".to_string()));

        assert!(ui.has_unsaved_changes);
        assert_eq!(ui.get_string("test.key", "default"), "value");
    }

    #[test]
    fn test_all_tabs() {
        let tabs = SettingsTab::all();
        assert_eq!(tabs.len(), 6);
        assert_eq!(tabs[0], SettingsTab::General);
        assert_eq!(tabs[5], SettingsTab::Advanced);
    }

    #[test]
    fn test_tab_names() {
        assert_eq!(SettingsTab::General.name(), "General");
        assert_eq!(SettingsTab::Privacy.name(), "Privacy");
    }
}
