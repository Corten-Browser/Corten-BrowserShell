//! Unit tests for UiChrome component
//!
//! Following TDD: Write failing tests first (RED), then implement (GREEN), then refactor

use shared_types::{KeyboardShortcut, TabId};
use ui_chrome::UiChrome;

#[test]
fn test_ui_chrome_creation() {
    // Given no prior state
    // When creating a new UiChrome instance
    // Then it should have an empty address bar and one default tab

    // When
    let chrome = UiChrome::new();

    // Then
    assert_eq!(chrome.address_bar_text(), "");
    assert_eq!(chrome.tab_count(), 1);
    assert_eq!(chrome.active_tab_index(), 0);
}

#[test]
fn test_update_tab_title() {
    // Given a UiChrome instance with a tab
    // When updating the tab's title
    // Then the tab's title should be updated

    // Given
    let mut chrome = UiChrome::new();
    let tab_id = chrome.get_tab_id(0).expect("Tab should exist");

    // When
    let result = chrome.update_tab_title(tab_id, "New Title".to_string());

    // Then
    assert!(result.is_ok());
    assert_eq!(chrome.get_tab_title(tab_id).unwrap(), "New Title");
}

#[test]
fn test_update_tab_title_nonexistent_tab() {
    // Given a UiChrome instance
    // When updating a non-existent tab's title
    // Then it should return an error

    // Given
    let mut chrome = UiChrome::new();
    let fake_tab_id = TabId::new(); // Random ID that doesn't exist

    // When
    let result = chrome.update_tab_title(fake_tab_id, "Title".to_string());

    // Then
    assert!(result.is_err());
}

#[test]
fn test_update_loading_state() {
    // Given a UiChrome instance with a tab
    // When updating the tab's loading state
    // Then the loading state should be updated

    // Given
    let mut chrome = UiChrome::new();
    let tab_id = chrome.get_tab_id(0).expect("Tab should exist");

    // When
    chrome.update_loading_state(tab_id, true).expect("Should succeed");

    // Then
    assert_eq!(chrome.is_tab_loading(tab_id).unwrap(), true);

    // When
    chrome.update_loading_state(tab_id, false).expect("Should succeed");

    // Then
    assert_eq!(chrome.is_tab_loading(tab_id).unwrap(), false);
}

#[test]
fn test_update_loading_state_nonexistent_tab() {
    // Given a UiChrome instance
    // When updating a non-existent tab's loading state
    // Then it should return an error

    // Given
    let mut chrome = UiChrome::new();
    let fake_tab_id = TabId::new();

    // When
    let result = chrome.update_loading_state(fake_tab_id, true);

    // Then
    assert!(result.is_err());
}

#[test]
fn test_handle_address_bar_input() {
    // Given a UiChrome instance
    // When handling address bar input
    // Then the address bar text should be updated

    // Given
    let mut chrome = UiChrome::new();

    // When
    let result = chrome.handle_address_bar_input("https://example.com".to_string());

    // Then
    assert!(result.is_ok());
    assert_eq!(chrome.address_bar_text(), "https://example.com");
}

#[test]
fn test_handle_keyboard_shortcut_ctrl_t() {
    // Given a UiChrome instance with one tab
    // When handling Ctrl+T (new tab) shortcut
    // Then a new tab should be created

    // Given
    let mut chrome = UiChrome::new();
    let initial_count = chrome.tab_count();

    // When
    let result = chrome.handle_keyboard_shortcut(KeyboardShortcut::CtrlT);

    // Then
    assert!(result.is_ok());
    assert_eq!(chrome.tab_count(), initial_count + 1);
}

#[test]
fn test_handle_keyboard_shortcut_ctrl_w() {
    // Given a UiChrome instance with multiple tabs
    // When handling Ctrl+W (close tab) shortcut
    // Then the active tab should be closed

    // Given
    let mut chrome = UiChrome::new();
    chrome.handle_keyboard_shortcut(KeyboardShortcut::CtrlT).expect("Create second tab");
    let initial_count = chrome.tab_count();

    // When
    let result = chrome.handle_keyboard_shortcut(KeyboardShortcut::CtrlW);

    // Then
    assert!(result.is_ok());
    assert_eq!(chrome.tab_count(), initial_count - 1);
}

#[test]
fn test_handle_keyboard_shortcut_ctrl_w_last_tab() {
    // Given a UiChrome instance with only one tab
    // When handling Ctrl+W (close tab) shortcut
    // Then it should not close the last tab (return error)

    // Given
    let mut chrome = UiChrome::new();

    // When
    let result = chrome.handle_keyboard_shortcut(KeyboardShortcut::CtrlW);

    // Then
    assert!(result.is_err());
    assert_eq!(chrome.tab_count(), 1);
}

#[test]
fn test_handle_keyboard_shortcut_ctrl_l() {
    // Given a UiChrome instance
    // When handling Ctrl+L (focus address bar) shortcut
    // Then the address bar should be marked as focused

    // Given
    let mut chrome = UiChrome::new();

    // When
    let result = chrome.handle_keyboard_shortcut(KeyboardShortcut::CtrlL);

    // Then
    assert!(result.is_ok());
    assert!(chrome.is_address_bar_focused());
}

#[test]
fn test_handle_keyboard_shortcut_f5() {
    // Given a UiChrome instance with a tab
    // When handling F5 (reload) shortcut
    // Then the reload action should be triggered

    // Given
    let mut chrome = UiChrome::new();

    // When
    let result = chrome.handle_keyboard_shortcut(KeyboardShortcut::F5);

    // Then
    assert!(result.is_ok());
    // The reload action is delegated to other components via message bus
}

#[test]
fn test_add_new_tab() {
    // Given a UiChrome instance
    // When adding a new tab
    // Then the tab count should increase and the new tab should be active

    // Given
    let mut chrome = UiChrome::new();
    let initial_count = chrome.tab_count();

    // When
    let tab_id = chrome.add_tab("New Tab".to_string());

    // Then
    assert_eq!(chrome.tab_count(), initial_count + 1);
    assert_eq!(chrome.active_tab_id(), Some(tab_id));
    assert_eq!(chrome.get_tab_title(tab_id).unwrap(), "New Tab");
}

#[test]
fn test_set_active_tab() {
    // Given a UiChrome instance with multiple tabs
    // When setting a specific tab as active
    // Then that tab should become the active tab

    // Given
    let mut chrome = UiChrome::new();
    let tab1_id = chrome.get_tab_id(0).expect("Tab 0 exists");
    let tab2_id = chrome.add_tab("Tab 2".to_string());

    // When
    chrome.set_active_tab(tab1_id).expect("Should set active tab");

    // Then
    assert_eq!(chrome.active_tab_id(), Some(tab1_id));

    // When
    chrome.set_active_tab(tab2_id).expect("Should set active tab");

    // Then
    assert_eq!(chrome.active_tab_id(), Some(tab2_id));
}
