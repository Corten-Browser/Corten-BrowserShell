// @validates: REQ-UI-005
//! Unit tests for Theme manager

use ui_chrome::theme::{ThemeManager, Theme, ThemeColors};

#[test]
fn test_theme_manager_creation() {
    let manager = ThemeManager::new();
    assert_eq!(manager.current_theme(), Theme::Light);
}

#[test]
fn test_theme_manager_set_theme() {
    let mut manager = ThemeManager::new();
    manager.set_theme(Theme::Dark);
    assert_eq!(manager.current_theme(), Theme::Dark);
}

#[test]
fn test_theme_manager_auto_theme() {
    let mut manager = ThemeManager::new();
    manager.set_theme(Theme::Auto);
    assert_eq!(manager.current_theme(), Theme::Auto);
}

#[test]
fn test_theme_manager_get_colors_light() {
    let mut manager = ThemeManager::new();
    manager.set_theme(Theme::Light);

    let colors = manager.get_colors();
    assert_eq!(colors.background, "#FFFFFF");
    assert_eq!(colors.foreground, "#000000");
}

#[test]
fn test_theme_manager_get_colors_dark() {
    let mut manager = ThemeManager::new();
    manager.set_theme(Theme::Dark);

    let colors = manager.get_colors();
    assert_eq!(colors.background, "#1E1E1E");
    assert_eq!(colors.foreground, "#FFFFFF");
}

#[test]
fn test_theme_manager_detect_system_theme() {
    let manager = ThemeManager::new();
    let detected = manager.detect_system_theme();

    // Should return either Light or Dark (never Auto)
    assert!(matches!(detected, Theme::Light | Theme::Dark));
}

#[test]
fn test_theme_manager_resolve_auto_theme() {
    let mut manager = ThemeManager::new();
    manager.set_theme(Theme::Auto);

    let resolved = manager.resolve_theme();
    assert!(matches!(resolved, Theme::Light | Theme::Dark));
}

#[test]
fn test_theme_manager_custom_colors() {
    let colors = ThemeColors {
        background: "#FFFFFF".to_string(),
        foreground: "#000000".to_string(),
        accent: "#0078D4".to_string(),
        border: "#E0E0E0".to_string(),
    };

    assert_eq!(colors.accent, "#0078D4");
    assert_eq!(colors.border, "#E0E0E0");
}

#[test]
fn test_theme_variants() {
    assert_eq!(Theme::Light, Theme::Light);
    assert_ne!(Theme::Light, Theme::Dark);
    assert_ne!(Theme::Dark, Theme::Auto);
}

#[test]
fn test_theme_manager_toggle() {
    let mut manager = ThemeManager::new();
    manager.set_theme(Theme::Light);

    manager.toggle_theme();
    assert_eq!(manager.current_theme(), Theme::Dark);

    manager.toggle_theme();
    assert_eq!(manager.current_theme(), Theme::Light);
}
