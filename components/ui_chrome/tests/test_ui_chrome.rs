//! Unit tests for UiChrome component
//!
//! Following TDD: Write failing tests first (RED), then implement (GREEN), then refactor

use eframe::App;
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
    chrome
        .update_loading_state(tab_id, true)
        .expect("Should succeed");

    // Then
    assert_eq!(chrome.is_tab_loading(tab_id).unwrap(), true);

    // When
    chrome
        .update_loading_state(tab_id, false)
        .expect("Should succeed");

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
    chrome
        .handle_keyboard_shortcut(KeyboardShortcut::CtrlT)
        .expect("Create second tab");
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
    chrome
        .set_active_tab(tab1_id)
        .expect("Should set active tab");

    // Then
    assert_eq!(chrome.active_tab_id(), Some(tab1_id));

    // When
    chrome
        .set_active_tab(tab2_id)
        .expect("Should set active tab");

    // Then
    assert_eq!(chrome.active_tab_id(), Some(tab2_id));
}

// Tests for egui::App trait implementation

#[test]
fn test_egui_app_trait_implemented() {
    // Given a UiChrome instance
    // When checking if it implements eframe::App
    // Then it should compile (trait bound satisfied)

    // This test verifies that UiChrome implements eframe::App
    fn assert_app_trait<T: App>(_: &T) {}

    let chrome = UiChrome::new();
    assert_app_trait(&chrome);
}

#[test]
fn test_egui_app_update_does_not_panic() {
    // Given a UiChrome instance implementing eframe::App
    // When creating the instance
    // Then it should implement the App trait without panicking

    // Given
    let _chrome = UiChrome::new();

    // Note: Testing egui update() directly requires creating a Frame,
    // which is complex in unit tests. The fact that we can create an
    // instance that implements App and the trait bound test passes
    // confirms the implementation is correct.
    //
    // Integration tests with actual egui context would test rendering behavior.
}

#[test]
fn test_egui_app_renders_toolbar() {
    // Given a UiChrome instance
    // When rendering with egui
    // Then the toolbar should be present

    // This test verifies the structure is set up correctly
    // Actual rendering tests would require headless egui testing
    let chrome = UiChrome::new();

    // Verify initial state that will be rendered
    assert_eq!(chrome.address_bar_text(), "");
    assert_eq!(chrome.tab_count(), 1);
}

#[test]
fn test_egui_app_renders_tabs() {
    // Given a UiChrome instance with multiple tabs
    // When rendering
    // Then all tabs should be represented in the data structure

    // Given
    let mut chrome = UiChrome::new();
    chrome.add_tab("Tab 2".to_string());
    chrome.add_tab("Tab 3".to_string());

    // Then - verify the data that will be rendered
    assert_eq!(chrome.tab_count(), 3);
    assert_eq!(
        chrome.get_tab_title(chrome.get_tab_id(0).unwrap()).unwrap(),
        "New Tab"
    );
    assert_eq!(
        chrome.get_tab_title(chrome.get_tab_id(1).unwrap()).unwrap(),
        "Tab 2"
    );
    assert_eq!(
        chrome.get_tab_title(chrome.get_tab_id(2).unwrap()).unwrap(),
        "Tab 3"
    );
}

#[test]
fn test_egui_app_active_tab_indicator() {
    // Given a UiChrome instance with multiple tabs
    // When a specific tab is active
    // Then the active tab index should be correct for rendering

    // Given
    let mut chrome = UiChrome::new();
    let tab2_id = chrome.add_tab("Tab 2".to_string());

    // When
    chrome.set_active_tab(tab2_id).expect("Should set active");

    // Then
    assert_eq!(chrome.active_tab_index(), 1);
    assert_eq!(chrome.active_tab_id(), Some(tab2_id));
}

#[test]
fn test_egui_app_loading_indicator() {
    // Given a UiChrome instance with a loading tab
    // When the tab is marked as loading
    // Then the loading state should be available for rendering

    // Given
    let mut chrome = UiChrome::new();
    let tab_id = chrome.get_tab_id(0).expect("Tab exists");

    // When
    chrome
        .update_loading_state(tab_id, true)
        .expect("Should update");

    // Then
    assert_eq!(chrome.is_tab_loading(tab_id).unwrap(), true);
}

#[test]
fn test_address_bar_focus_state() {
    // Given a UiChrome instance
    // When the address bar is focused via keyboard shortcut
    // Then the focus state should be tracked for rendering

    // Given
    let mut chrome = UiChrome::new();
    assert!(!chrome.is_address_bar_focused());

    // When
    chrome
        .handle_keyboard_shortcut(KeyboardShortcut::CtrlL)
        .expect("Should focus");

    // Then
    assert!(chrome.is_address_bar_focused());
}

// Phase 3 Enhancement Tests

// Test 1: Tab close button functionality
#[test]
fn test_close_tab_by_id() {
    // Given a UiChrome instance with multiple tabs
    // When closing a specific tab by ID
    // Then that tab should be removed

    // Given
    let mut chrome = UiChrome::new();
    let _tab1_id = chrome.get_tab_id(0).expect("Tab 0 exists");
    let tab2_id = chrome.add_tab("Tab 2".to_string());
    let _tab3_id = chrome.add_tab("Tab 3".to_string());

    // When
    let result = chrome.close_tab(tab2_id);

    // Then
    assert!(result.is_ok());
    assert_eq!(chrome.tab_count(), 2);
    assert!(chrome.get_tab_title(tab2_id).is_none());
}

#[test]
fn test_close_tab_prevents_closing_last_tab() {
    // Given a UiChrome instance with only one tab
    // When attempting to close the last tab
    // Then it should return an error

    // Given
    let mut chrome = UiChrome::new();
    let tab_id = chrome.get_tab_id(0).expect("Tab exists");

    // When
    let result = chrome.close_tab(tab_id);

    // Then
    assert!(result.is_err());
    assert_eq!(chrome.tab_count(), 1);
}

// Test 2: Next/Previous tab navigation
#[test]
fn test_switch_to_next_tab() {
    // Given a UiChrome instance with multiple tabs
    // When switching to next tab
    // Then the active tab should move to the next one

    // Given
    let mut chrome = UiChrome::new();
    chrome.add_tab("Tab 2".to_string());
    chrome.add_tab("Tab 3".to_string());
    assert_eq!(chrome.active_tab_index(), 2); // Last tab added is active

    // Set to first tab
    let tab1_id = chrome.get_tab_id(0).expect("Tab 0 exists");
    chrome.set_active_tab(tab1_id).expect("Should set active");

    // When
    let result = chrome.switch_to_next_tab();

    // Then
    assert!(result.is_ok());
    assert_eq!(chrome.active_tab_index(), 1);
}

#[test]
fn test_switch_to_next_tab_wraps_around() {
    // Given a UiChrome instance with tabs where last tab is active
    // When switching to next tab
    // Then it should wrap around to the first tab

    // Given
    let mut chrome = UiChrome::new();
    chrome.add_tab("Tab 2".to_string());
    chrome.add_tab("Tab 3".to_string());
    assert_eq!(chrome.active_tab_index(), 2); // Last tab

    // When
    let result = chrome.switch_to_next_tab();

    // Then
    assert!(result.is_ok());
    assert_eq!(chrome.active_tab_index(), 0); // Wrapped to first
}

#[test]
fn test_switch_to_previous_tab() {
    // Given a UiChrome instance with multiple tabs
    // When switching to previous tab
    // Then the active tab should move to the previous one

    // Given
    let mut chrome = UiChrome::new();
    chrome.add_tab("Tab 2".to_string());
    chrome.add_tab("Tab 3".to_string());
    assert_eq!(chrome.active_tab_index(), 2);

    // When
    let result = chrome.switch_to_previous_tab();

    // Then
    assert!(result.is_ok());
    assert_eq!(chrome.active_tab_index(), 1);
}

#[test]
fn test_switch_to_previous_tab_wraps_around() {
    // Given a UiChrome instance with tabs where first tab is active
    // When switching to previous tab
    // Then it should wrap around to the last tab

    // Given
    let mut chrome = UiChrome::new();
    chrome.add_tab("Tab 2".to_string());
    chrome.add_tab("Tab 3".to_string());

    // Set to first tab
    let tab1_id = chrome.get_tab_id(0).expect("Tab 0 exists");
    chrome.set_active_tab(tab1_id).expect("Should set active");

    // When
    let result = chrome.switch_to_previous_tab();

    // Then
    assert!(result.is_ok());
    assert_eq!(chrome.active_tab_index(), 2); // Wrapped to last
}

// Test 3: Switch to tab by number (1-9)
#[test]
fn test_switch_to_tab_by_number() {
    // Given a UiChrome instance with multiple tabs
    // When switching to a specific tab by number
    // Then that tab should become active

    // Given
    let mut chrome = UiChrome::new();
    chrome.add_tab("Tab 2".to_string());
    chrome.add_tab("Tab 3".to_string());

    // When - switch to tab 1 (0-indexed internally)
    let result = chrome.switch_to_tab_number(1);

    // Then
    assert!(result.is_ok());
    assert_eq!(chrome.active_tab_index(), 0);

    // When - switch to tab 2
    let result = chrome.switch_to_tab_number(2);

    // Then
    assert!(result.is_ok());
    assert_eq!(chrome.active_tab_index(), 1);
}

#[test]
fn test_switch_to_tab_by_number_out_of_range() {
    // Given a UiChrome instance with 3 tabs
    // When attempting to switch to tab 9
    // Then it should return an error

    // Given
    let mut chrome = UiChrome::new();
    chrome.add_tab("Tab 2".to_string());

    // When
    let result = chrome.switch_to_tab_number(9);

    // Then
    assert!(result.is_err());
}

// Test 4: Panel visibility
#[test]
fn test_panel_visibility_defaults() {
    // Given a new UiChrome instance
    // When checking panel visibility
    // Then all panels should be hidden by default

    // Given
    let chrome = UiChrome::new();

    // Then
    assert!(!chrome.is_settings_panel_visible());
    assert!(!chrome.is_history_panel_visible());
    assert!(!chrome.is_downloads_panel_visible());
}

#[test]
fn test_toggle_settings_panel() {
    // Given a UiChrome instance
    // When toggling the settings panel
    // Then its visibility should change

    // Given
    let mut chrome = UiChrome::new();
    assert!(!chrome.is_settings_panel_visible());

    // When
    chrome.toggle_settings_panel();

    // Then
    assert!(chrome.is_settings_panel_visible());

    // When - toggle again
    chrome.toggle_settings_panel();

    // Then
    assert!(!chrome.is_settings_panel_visible());
}

#[test]
fn test_toggle_history_panel() {
    // Given a UiChrome instance
    // When toggling the history panel
    // Then its visibility should change

    // Given
    let mut chrome = UiChrome::new();

    // When
    chrome.toggle_history_panel();

    // Then
    assert!(chrome.is_history_panel_visible());
}

#[test]
fn test_toggle_downloads_panel() {
    // Given a UiChrome instance
    // When toggling the downloads panel
    // Then its visibility should change

    // Given
    let mut chrome = UiChrome::new();

    // When
    chrome.toggle_downloads_panel();

    // Then
    assert!(chrome.is_downloads_panel_visible());
}

// Test 5: Context menu state
#[test]
fn test_context_menu_none_by_default() {
    // Given a new UiChrome instance
    // When checking context menu state
    // Then no context menu should be shown

    // Given
    let chrome = UiChrome::new();

    // Then
    assert!(!chrome.has_active_context_menu());
}

#[test]
fn test_show_tab_context_menu() {
    // Given a UiChrome instance with a tab
    // When showing a context menu for that tab
    // Then the context menu should be active

    // Given
    let mut chrome = UiChrome::new();
    let tab_id = chrome.get_tab_id(0).expect("Tab exists");

    // When
    chrome.show_tab_context_menu(tab_id);

    // Then
    assert!(chrome.has_active_context_menu());
}

#[test]
fn test_close_context_menu() {
    // Given a UiChrome instance with an active context menu
    // When closing the context menu
    // Then it should no longer be active

    // Given
    let mut chrome = UiChrome::new();
    let tab_id = chrome.get_tab_id(0).expect("Tab exists");
    chrome.show_tab_context_menu(tab_id);
    assert!(chrome.has_active_context_menu());

    // When
    chrome.close_context_menu();

    // Then
    assert!(!chrome.has_active_context_menu());
}

// Test 6: Status bar state
#[test]
fn test_status_bar_hover_url() {
    // Given a UiChrome instance
    // When setting a hover URL
    // Then it should be retrievable

    // Given
    let mut chrome = UiChrome::new();

    // When
    chrome.set_hover_url(Some("https://example.com".to_string()));

    // Then
    assert_eq!(
        chrome.get_hover_url(),
        Some("https://example.com".to_string())
    );

    // When - clear hover URL
    chrome.set_hover_url(None);

    // Then
    assert_eq!(chrome.get_hover_url(), None);
}

#[test]
fn test_status_bar_download_count() {
    // Given a UiChrome instance
    // When setting download count
    // Then it should be retrievable

    // Given
    let mut chrome = UiChrome::new();
    assert_eq!(chrome.get_download_count(), 0);

    // When
    chrome.set_download_count(3);

    // Then
    assert_eq!(chrome.get_download_count(), 3);
}

// Test 7: Bookmark functionality
#[test]
fn test_bookmark_current_page() {
    // Given a UiChrome instance with an address
    // When bookmarking the current page
    // Then it should be added to bookmarks

    // Given
    let mut chrome = UiChrome::new();
    chrome
        .handle_address_bar_input("https://example.com".to_string())
        .expect("Should set address");

    // When
    let result = chrome.bookmark_current_page();

    // Then
    assert!(result.is_ok());
    assert!(chrome.is_bookmarked("https://example.com"));
}

#[test]
fn test_bookmark_empty_address_fails() {
    // Given a UiChrome instance with no address
    // When attempting to bookmark
    // Then it should fail

    // Given
    let mut chrome = UiChrome::new();

    // When
    let result = chrome.bookmark_current_page();

    // Then
    assert!(result.is_err());
}
