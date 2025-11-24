//! Theme system for the browser UI
//!
//! Provides light, dark, and auto theme modes with customizable colors and fonts.
//! Themes can be persisted via settings_manager and applied at runtime without restart.

use egui::Color32;
use serde::{Deserialize, Serialize};

/// Theme mode selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ThemeMode {
    /// Light theme (default)
    #[default]
    Light,
    /// Dark theme
    Dark,
    /// Automatically follow system preference
    Auto,
}

impl ThemeMode {
    /// Convert to string representation (for settings integration)
    pub fn as_str(&self) -> &'static str {
        match self {
            ThemeMode::Light => "light",
            ThemeMode::Dark => "dark",
            ThemeMode::Auto => "auto",
        }
    }
}

impl std::str::FromStr for ThemeMode {
    type Err = std::convert::Infallible;

    /// Parse theme mode from string (for settings integration)
    ///
    /// This implementation never fails - unrecognized strings default to Light.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "dark" => ThemeMode::Dark,
            "auto" | "system" => ThemeMode::Auto,
            _ => ThemeMode::Light,
        })
    }
}

/// Browser theme configuration
///
/// Contains all customizable visual properties for the browser UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    /// Current theme mode
    pub mode: ThemeMode,
    /// Main background color
    pub background: Color32,
    /// Main text/foreground color
    pub foreground: Color32,
    /// Accent color for highlights, buttons, links
    pub accent: Color32,
    /// Toolbar background color
    pub toolbar_bg: Color32,
    /// Active tab background color
    pub tab_active_bg: Color32,
    /// Inactive tab background color
    pub tab_inactive_bg: Color32,
    /// Font family name
    pub font_family: String,
    /// Base font size in points
    pub font_size: f32,
    /// Border radius for UI elements
    pub border_radius: f32,
}

impl Default for Theme {
    fn default() -> Self {
        Self::light()
    }
}

impl Theme {
    /// Create the default light theme
    pub fn light() -> Self {
        Self {
            mode: ThemeMode::Light,
            background: Color32::from_rgb(255, 255, 255),
            foreground: Color32::from_rgb(30, 30, 30),
            accent: Color32::from_rgb(0, 120, 212),
            toolbar_bg: Color32::from_rgb(243, 243, 243),
            tab_active_bg: Color32::from_rgb(255, 255, 255),
            tab_inactive_bg: Color32::from_rgb(230, 230, 230),
            font_family: "system-ui".to_string(),
            font_size: 14.0,
            border_radius: 4.0,
        }
    }

    /// Create the default dark theme
    pub fn dark() -> Self {
        Self {
            mode: ThemeMode::Dark,
            background: Color32::from_rgb(30, 30, 30),
            foreground: Color32::from_rgb(230, 230, 230),
            accent: Color32::from_rgb(96, 165, 250),
            toolbar_bg: Color32::from_rgb(45, 45, 45),
            tab_active_bg: Color32::from_rgb(60, 60, 60),
            tab_inactive_bg: Color32::from_rgb(40, 40, 40),
            font_family: "system-ui".to_string(),
            font_size: 14.0,
            border_radius: 4.0,
        }
    }

    /// Create a theme based on mode, using system preference for Auto
    pub fn for_mode(mode: ThemeMode) -> Self {
        match mode {
            ThemeMode::Light => Self::light(),
            ThemeMode::Dark => Self::dark(),
            ThemeMode::Auto => {
                if detect_system_dark_mode() {
                    Self::dark()
                } else {
                    Self::light()
                }
            }
        }
    }

    /// Create a custom theme with the specified accent color
    pub fn with_accent(mut self, accent: Color32) -> Self {
        self.accent = accent;
        self
    }

    /// Create a custom theme with the specified font settings
    pub fn with_font(mut self, family: String, size: f32) -> Self {
        self.font_family = family;
        self.font_size = size;
        self
    }

    /// Create a custom theme with the specified border radius
    pub fn with_border_radius(mut self, radius: f32) -> Self {
        self.border_radius = radius;
        self
    }

    /// Apply this theme to an egui context
    ///
    /// Updates the egui visuals and style to match this theme's colors and settings.
    pub fn apply(&self, ctx: &egui::Context) {
        let mut visuals = if matches!(self.mode, ThemeMode::Dark)
            || (matches!(self.mode, ThemeMode::Auto) && detect_system_dark_mode())
        {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };

        // Apply custom colors
        visuals.override_text_color = Some(self.foreground);
        visuals.panel_fill = self.background;
        visuals.window_fill = self.background;
        visuals.extreme_bg_color = self.toolbar_bg;
        visuals.faint_bg_color = self.tab_inactive_bg;

        // Apply accent color to widgets
        visuals.selection.bg_fill = self.accent.linear_multiply(0.3);
        visuals.selection.stroke.color = self.accent;
        visuals.hyperlink_color = self.accent;

        // Widget styling
        visuals.widgets.active.bg_fill = self.accent;
        visuals.widgets.hovered.bg_fill = self.accent.linear_multiply(0.8);

        // Border radius
        visuals.window_rounding = egui::Rounding::same(self.border_radius);
        visuals.menu_rounding = egui::Rounding::same(self.border_radius);

        ctx.set_visuals(visuals);

        // Apply font size through style
        let mut style = (*ctx.style()).clone();
        style.text_styles.iter_mut().for_each(|(_, font_id)| {
            font_id.size = self.font_size;
        });
        style.visuals = ctx.style().visuals.clone();
        ctx.set_style(style);
    }

    /// Check if this theme is effectively dark (either Dark mode or Auto with system dark)
    pub fn is_dark(&self) -> bool {
        matches!(self.mode, ThemeMode::Dark)
            || (matches!(self.mode, ThemeMode::Auto) && detect_system_dark_mode())
    }

    /// Resolve the current effective theme based on mode
    ///
    /// For Auto mode, this checks the system preference and returns
    /// the appropriate light or dark theme.
    pub fn resolve(&self) -> Theme {
        if matches!(self.mode, ThemeMode::Auto) {
            if detect_system_dark_mode() {
                let mut theme = Theme::dark();
                theme.mode = ThemeMode::Auto;
                theme.accent = self.accent;
                theme.font_family = self.font_family.clone();
                theme.font_size = self.font_size;
                theme.border_radius = self.border_radius;
                theme
            } else {
                let mut theme = Theme::light();
                theme.mode = ThemeMode::Auto;
                theme.accent = self.accent;
                theme.font_family = self.font_family.clone();
                theme.font_size = self.font_size;
                theme.border_radius = self.border_radius;
                theme
            }
        } else {
            self.clone()
        }
    }
}

/// Detect if the system prefers dark mode
///
/// Platform-specific implementation:
/// - Linux: Checks GTK/GNOME settings via gsettings or environment variables
/// - macOS: Uses defaults command to check AppleInterfaceStyle
/// - Windows: Checks registry for AppsUseLightTheme
/// - Falls back to light theme if detection fails
pub fn detect_system_dark_mode() -> bool {
    #[cfg(target_os = "linux")]
    {
        detect_dark_mode_linux()
    }

    #[cfg(target_os = "macos")]
    {
        detect_dark_mode_macos()
    }

    #[cfg(target_os = "windows")]
    {
        detect_dark_mode_windows()
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        false
    }
}

#[cfg(target_os = "linux")]
fn detect_dark_mode_linux() -> bool {
    // Check XDG_CURRENT_DESKTOP for desktop environment
    // Then try gsettings for GNOME/GTK
    if let Ok(output) = std::process::Command::new("gsettings")
        .args(["get", "org.gnome.desktop.interface", "color-scheme"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("prefer-dark") || stdout.contains("dark") {
            return true;
        }
    }

    // Try reading GTK theme name as fallback
    if let Ok(output) = std::process::Command::new("gsettings")
        .args(["get", "org.gnome.desktop.interface", "gtk-theme"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.to_lowercase().contains("dark") {
            return true;
        }
    }

    // Check environment variable (some DEs set this)
    if let Ok(val) = std::env::var("GTK_THEME") {
        if val.to_lowercase().contains("dark") {
            return true;
        }
    }

    false
}

#[cfg(target_os = "macos")]
fn detect_dark_mode_macos() -> bool {
    // Use defaults command to check system appearance
    if let Ok(output) = std::process::Command::new("defaults")
        .args(["read", "-g", "AppleInterfaceStyle"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        return stdout.trim().eq_ignore_ascii_case("dark");
    }
    false
}

#[cfg(target_os = "windows")]
fn detect_dark_mode_windows() -> bool {
    // Check Windows registry for apps light theme setting
    // Path: HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize
    // Key: AppsUseLightTheme (0 = dark, 1 = light)
    if let Ok(output) = std::process::Command::new("reg")
        .args([
            "query",
            r"HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize",
            "/v",
            "AppsUseLightTheme",
        ])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // The output contains "0x0" for dark mode, "0x1" for light mode
        if stdout.contains("0x0") {
            return true;
        }
    }
    false
}

/// Theme manager for handling runtime theme changes
///
/// Provides a higher-level interface for managing themes, including
/// caching and change detection.
#[derive(Debug, Clone)]
pub struct ThemeManager {
    /// Current active theme
    current_theme: Theme,
    /// Whether the theme has changed since last apply
    needs_apply: bool,
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ThemeManager {
    /// Create a new theme manager with the default light theme
    pub fn new() -> Self {
        Self {
            current_theme: Theme::default(),
            needs_apply: true,
        }
    }

    /// Create a theme manager with a specific theme
    pub fn with_theme(theme: Theme) -> Self {
        Self {
            current_theme: theme,
            needs_apply: true,
        }
    }

    /// Get the current theme
    pub fn theme(&self) -> &Theme {
        &self.current_theme
    }

    /// Get the current theme mode
    pub fn mode(&self) -> ThemeMode {
        self.current_theme.mode
    }

    /// Set the theme mode
    pub fn set_mode(&mut self, mode: ThemeMode) {
        if self.current_theme.mode != mode {
            self.current_theme = Theme::for_mode(mode)
                .with_accent(self.current_theme.accent)
                .with_font(
                    self.current_theme.font_family.clone(),
                    self.current_theme.font_size,
                )
                .with_border_radius(self.current_theme.border_radius);
            self.current_theme.mode = mode;
            self.needs_apply = true;
        }
    }

    /// Set the accent color
    pub fn set_accent(&mut self, accent: Color32) {
        if self.current_theme.accent != accent {
            self.current_theme.accent = accent;
            self.needs_apply = true;
        }
    }

    /// Set the font settings
    pub fn set_font(&mut self, family: String, size: f32) {
        if self.current_theme.font_family != family || self.current_theme.font_size != size {
            self.current_theme.font_family = family;
            self.current_theme.font_size = size;
            self.needs_apply = true;
        }
    }

    /// Set the border radius
    pub fn set_border_radius(&mut self, radius: f32) {
        if (self.current_theme.border_radius - radius).abs() > f32::EPSILON {
            self.current_theme.border_radius = radius;
            self.needs_apply = true;
        }
    }

    /// Apply the theme to the egui context if needed
    ///
    /// Returns true if the theme was applied, false if no changes were needed.
    pub fn apply_if_needed(&mut self, ctx: &egui::Context) -> bool {
        if self.needs_apply {
            self.current_theme.apply(ctx);
            self.needs_apply = false;
            true
        } else {
            false
        }
    }

    /// Force apply the theme to the egui context
    pub fn apply(&mut self, ctx: &egui::Context) {
        self.current_theme.apply(ctx);
        self.needs_apply = false;
    }

    /// Check if the theme needs to be reapplied
    ///
    /// This is useful for Auto mode where the system preference may have changed.
    pub fn check_system_change(&mut self) {
        if matches!(self.current_theme.mode, ThemeMode::Auto) {
            let current_is_dark = self.current_theme.is_dark();
            let system_is_dark = detect_system_dark_mode();
            if current_is_dark != system_is_dark {
                self.needs_apply = true;
                // Update the theme colors based on new system preference
                let base = if system_is_dark {
                    Theme::dark()
                } else {
                    Theme::light()
                };
                self.current_theme.background = base.background;
                self.current_theme.foreground = base.foreground;
                self.current_theme.toolbar_bg = base.toolbar_bg;
                self.current_theme.tab_active_bg = base.tab_active_bg;
                self.current_theme.tab_inactive_bg = base.tab_inactive_bg;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_mode_default() {
        let mode = ThemeMode::default();
        assert_eq!(mode, ThemeMode::Light);
    }

    #[test]
    fn test_theme_mode_from_str() {
        use std::str::FromStr;
        assert_eq!(ThemeMode::from_str("light").unwrap(), ThemeMode::Light);
        assert_eq!(ThemeMode::from_str("Light").unwrap(), ThemeMode::Light);
        assert_eq!(ThemeMode::from_str("LIGHT").unwrap(), ThemeMode::Light);
        assert_eq!(ThemeMode::from_str("dark").unwrap(), ThemeMode::Dark);
        assert_eq!(ThemeMode::from_str("Dark").unwrap(), ThemeMode::Dark);
        assert_eq!(ThemeMode::from_str("auto").unwrap(), ThemeMode::Auto);
        assert_eq!(ThemeMode::from_str("system").unwrap(), ThemeMode::Auto);
        assert_eq!(ThemeMode::from_str("invalid").unwrap(), ThemeMode::Light);
    }

    #[test]
    fn test_theme_mode_as_str() {
        assert_eq!(ThemeMode::Light.as_str(), "light");
        assert_eq!(ThemeMode::Dark.as_str(), "dark");
        assert_eq!(ThemeMode::Auto.as_str(), "auto");
    }

    #[test]
    fn test_theme_light_defaults() {
        let theme = Theme::light();
        assert_eq!(theme.mode, ThemeMode::Light);
        assert_eq!(theme.background, Color32::from_rgb(255, 255, 255));
        assert_eq!(theme.foreground, Color32::from_rgb(30, 30, 30));
        assert_eq!(theme.font_size, 14.0);
        assert_eq!(theme.border_radius, 4.0);
    }

    #[test]
    fn test_theme_dark_defaults() {
        let theme = Theme::dark();
        assert_eq!(theme.mode, ThemeMode::Dark);
        assert_eq!(theme.background, Color32::from_rgb(30, 30, 30));
        assert_eq!(theme.foreground, Color32::from_rgb(230, 230, 230));
        assert_eq!(theme.font_size, 14.0);
        assert_eq!(theme.border_radius, 4.0);
    }

    #[test]
    fn test_theme_with_accent() {
        let theme = Theme::light().with_accent(Color32::RED);
        assert_eq!(theme.accent, Color32::RED);
        // Other properties should be unchanged
        assert_eq!(theme.mode, ThemeMode::Light);
        assert_eq!(theme.font_size, 14.0);
    }

    #[test]
    fn test_theme_with_font() {
        let theme = Theme::light().with_font("Arial".to_string(), 16.0);
        assert_eq!(theme.font_family, "Arial");
        assert_eq!(theme.font_size, 16.0);
    }

    #[test]
    fn test_theme_with_border_radius() {
        let theme = Theme::light().with_border_radius(8.0);
        assert_eq!(theme.border_radius, 8.0);
    }

    #[test]
    fn test_theme_is_dark() {
        let light = Theme::light();
        let dark = Theme::dark();
        assert!(!light.is_dark());
        assert!(dark.is_dark());
    }

    #[test]
    fn test_theme_for_mode_light() {
        let theme = Theme::for_mode(ThemeMode::Light);
        assert_eq!(theme.mode, ThemeMode::Light);
        assert_eq!(theme.background, Color32::from_rgb(255, 255, 255));
    }

    #[test]
    fn test_theme_for_mode_dark() {
        let theme = Theme::for_mode(ThemeMode::Dark);
        assert_eq!(theme.mode, ThemeMode::Dark);
        assert_eq!(theme.background, Color32::from_rgb(30, 30, 30));
    }

    #[test]
    fn test_theme_default() {
        let theme = Theme::default();
        assert_eq!(theme.mode, ThemeMode::Light);
    }

    #[test]
    fn test_theme_manager_new() {
        let manager = ThemeManager::new();
        assert_eq!(manager.mode(), ThemeMode::Light);
        assert!(manager.needs_apply);
    }

    #[test]
    fn test_theme_manager_with_theme() {
        let manager = ThemeManager::with_theme(Theme::dark());
        assert_eq!(manager.mode(), ThemeMode::Dark);
    }

    #[test]
    fn test_theme_manager_set_mode() {
        let mut manager = ThemeManager::new();
        manager.set_mode(ThemeMode::Dark);
        assert_eq!(manager.mode(), ThemeMode::Dark);
        assert!(manager.needs_apply);
    }

    #[test]
    fn test_theme_manager_set_mode_same() {
        let mut manager = ThemeManager::new();
        manager.needs_apply = false;
        manager.set_mode(ThemeMode::Light); // Same as current
        assert!(!manager.needs_apply); // Should not mark as needing apply
    }

    #[test]
    fn test_theme_manager_set_accent() {
        let mut manager = ThemeManager::new();
        manager.needs_apply = false;
        manager.set_accent(Color32::GREEN);
        assert_eq!(manager.theme().accent, Color32::GREEN);
        assert!(manager.needs_apply);
    }

    #[test]
    fn test_theme_manager_set_font() {
        let mut manager = ThemeManager::new();
        manager.needs_apply = false;
        manager.set_font("Helvetica".to_string(), 18.0);
        assert_eq!(manager.theme().font_family, "Helvetica");
        assert_eq!(manager.theme().font_size, 18.0);
        assert!(manager.needs_apply);
    }

    #[test]
    fn test_theme_manager_set_border_radius() {
        let mut manager = ThemeManager::new();
        manager.needs_apply = false;
        manager.set_border_radius(10.0);
        assert_eq!(manager.theme().border_radius, 10.0);
        assert!(manager.needs_apply);
    }

    #[test]
    fn test_theme_manager_preserves_custom_settings_on_mode_change() {
        let mut manager = ThemeManager::new();
        manager.set_accent(Color32::GREEN);
        manager.set_font("Monaco".to_string(), 12.0);
        manager.set_border_radius(6.0);

        manager.set_mode(ThemeMode::Dark);

        // Custom settings should be preserved
        assert_eq!(manager.theme().accent, Color32::GREEN);
        assert_eq!(manager.theme().font_family, "Monaco");
        assert_eq!(manager.theme().font_size, 12.0);
        assert_eq!(manager.theme().border_radius, 6.0);
        // Mode should be updated
        assert_eq!(manager.mode(), ThemeMode::Dark);
    }

    #[test]
    fn test_theme_resolve_light() {
        let theme = Theme::light();
        let resolved = theme.resolve();
        assert_eq!(resolved.mode, ThemeMode::Light);
        assert_eq!(resolved.background, Color32::from_rgb(255, 255, 255));
    }

    #[test]
    fn test_theme_resolve_dark() {
        let theme = Theme::dark();
        let resolved = theme.resolve();
        assert_eq!(resolved.mode, ThemeMode::Dark);
        assert_eq!(resolved.background, Color32::from_rgb(30, 30, 30));
    }

    #[test]
    fn test_theme_serialization() {
        let theme = Theme::light().with_accent(Color32::from_rgb(100, 150, 200));
        let serialized = serde_json::to_string(&theme).expect("Failed to serialize");
        let deserialized: Theme = serde_json::from_str(&serialized).expect("Failed to deserialize");
        assert_eq!(deserialized.mode, theme.mode);
        assert_eq!(deserialized.accent, theme.accent);
        assert_eq!(deserialized.font_family, theme.font_family);
        assert_eq!(deserialized.font_size, theme.font_size);
    }

    #[test]
    fn test_theme_mode_serialization() {
        let modes = vec![ThemeMode::Light, ThemeMode::Dark, ThemeMode::Auto];
        for mode in modes {
            let serialized = serde_json::to_string(&mode).expect("Failed to serialize");
            let deserialized: ThemeMode =
                serde_json::from_str(&serialized).expect("Failed to deserialize");
            assert_eq!(deserialized, mode);
        }
    }
}
