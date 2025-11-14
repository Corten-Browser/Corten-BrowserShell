// @validates: REQ-UI-003
//! Unit tests for Toolbar widget

use ui_chrome::widgets::toolbar::{Toolbar, ToolbarButton, ToolbarAction};

#[test]
fn test_toolbar_creation() {
    let toolbar = Toolbar::new();
    assert_eq!(toolbar.button_count(), 6); // back, forward, reload, stop, home, bookmarks
}

#[test]
fn test_toolbar_button_enabled_state() {
    let mut toolbar = Toolbar::new();

    // Back button should start disabled
    assert!(!toolbar.is_button_enabled(ToolbarButton::Back));

    // Enable back button
    toolbar.set_button_enabled(ToolbarButton::Back, true);
    assert!(toolbar.is_button_enabled(ToolbarButton::Back));
}

#[test]
fn test_toolbar_button_visible_state() {
    let mut toolbar = Toolbar::new();

    // All buttons should start visible
    assert!(toolbar.is_button_visible(ToolbarButton::Reload));

    // Hide reload button
    toolbar.set_button_visible(ToolbarButton::Reload, false);
    assert!(!toolbar.is_button_visible(ToolbarButton::Reload));
}

#[test]
fn test_toolbar_handle_click() {
    let mut toolbar = Toolbar::new();
    toolbar.set_button_enabled(ToolbarButton::Reload, true);

    let action = toolbar.handle_click(ToolbarButton::Reload);
    assert!(action.is_some());
    assert_eq!(action.unwrap(), ToolbarAction::Reload);
}

#[test]
fn test_toolbar_handle_click_disabled_button() {
    let mut toolbar = Toolbar::new();
    toolbar.set_button_enabled(ToolbarButton::Back, false);

    let action = toolbar.handle_click(ToolbarButton::Back);
    assert!(action.is_none()); // Disabled buttons produce no action
}

#[test]
fn test_toolbar_update_navigation_state() {
    let mut toolbar = Toolbar::new();

    toolbar.update_navigation_state(true, false, false);
    assert!(toolbar.is_button_enabled(ToolbarButton::Back));
    assert!(!toolbar.is_button_enabled(ToolbarButton::Forward));
    assert!(!toolbar.is_button_enabled(ToolbarButton::Stop));
}

#[test]
fn test_toolbar_set_loading_state() {
    let mut toolbar = Toolbar::new();

    toolbar.set_loading(true);
    assert!(toolbar.is_button_enabled(ToolbarButton::Stop));
    assert!(!toolbar.is_button_visible(ToolbarButton::Reload));

    toolbar.set_loading(false);
    assert!(!toolbar.is_button_enabled(ToolbarButton::Stop));
    assert!(toolbar.is_button_visible(ToolbarButton::Reload));
}

#[test]
fn test_toolbar_button_tooltip() {
    let toolbar = Toolbar::new();
    let tooltip = toolbar.get_button_tooltip(ToolbarButton::Back);
    assert_eq!(tooltip, "Go back");
}

#[test]
fn test_toolbar_all_buttons() {
    let toolbar = Toolbar::new();
    let buttons = toolbar.get_all_buttons();
    assert_eq!(buttons.len(), 6);
    assert!(buttons.contains(&ToolbarButton::Back));
    assert!(buttons.contains(&ToolbarButton::Forward));
    assert!(buttons.contains(&ToolbarButton::Reload));
}
